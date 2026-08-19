//! Level of Detail (LOD) aggregation for voxel storage
//! Aggregates distant regions for efficient simulation and rendering

use std::collections::HashMap;
use crate::coords::{RegionKey, ChunkKey, CHUNK_SIZE, REGION_BLOCK_SIZE};
use crate::chunk::PaletteChunk;
use crate::block::BlockId;
use lithos_engine_math::{Vec3, AABB};
use serde::{Serialize, Deserialize};

/// Material ID for LOD aggregation
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct MaterialId(pub u16);

impl MaterialId {
    pub fn from_block(block_id: BlockId) -> Self {
        // Simple mapping - in reality would come from block definition
        MaterialId(block_id.0)
    }
}

impl From<u16> for MaterialId {
    fn from(id: u16) -> Self {
        MaterialId(id)
    }
}

impl From<MaterialId> for u16 {
    fn from(id: MaterialId) -> u16 {
        id.0
    }
}

/// Aggregated region data for LOD
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LodAggregate {
    pub region: RegionKey,
    /// Material volumes in this region (m³)
    pub material_volumes: HashMap<MaterialId, f32>,
    /// Average temperature (Kelvin)
    pub avg_temperature: f32,
    /// Total thermal energy (Joules)
    pub total_thermal_energy: f32,
    /// Process throughputs (volume/tick)
    pub process_throughputs: HashMap<u32, f32>,
    /// Power consumption (Watts)
    pub power_consumption: f32,
    /// Heat generation (Watts)
    pub heat_generation: f32,
    /// Entity counts by type
    pub entity_counts: HashMap<u32, u32>,
    /// Logistics flow summary
    pub logistics_flows: LogisticsAggregate,
    /// Bounding box of non-air blocks
    pub bounds: Option<AABB>,
    /// Last update tick
    pub last_updated: u64,
}

impl LodAggregate {
    pub fn new(region: RegionKey) -> Self {
        Self {
            region,
            material_volumes: HashMap::new(),
            avg_temperature: 293.15, // 20°C
            total_thermal_energy: 0.0,
            process_throughputs: HashMap::new(),
            power_consumption: 0.0,
            heat_generation: 0.0,
            entity_counts: HashMap::new(),
            logistics_flows: LogisticsAggregate::default(),
            bounds: None,
            last_updated: 0,
        }
    }

    /// Accumulate data from a chunk
    pub fn accumulate_chunk(&mut self, chunk: &PaletteChunk) {
        if chunk.empty || chunk.uniform && chunk.uniform_block.block_id.is_air() {
            return;
        }

        let blocks = chunk.to_blocks();
        let voxel_volume = 1.0 / (CHUNK_SIZE as f32).powi(3); // Volume per voxel in region space

        for (i, block) in blocks.iter().enumerate() {
            if block.block_id.is_air() {
                continue;
            }

            // Material volume
            let mat_id = MaterialId::from_block(block.block_id);
            *self.material_volumes.entry(mat_id).or_insert(0.0) += voxel_volume;

            // Update bounds
            let (x, y, z) = crate::chunk::block_coords(i);
            let pos = Vec3::new(x as f32, y as f32, z as f32);
            self.bounds = Some(match self.bounds {
                Some(mut b) => {
                    b.merge(&AABB::from_center_half_extents(pos, Vec3::splat(0.5)));
                    b
                },
                None => AABB::from_center_half_extents(pos, Vec3::splat(0.5)),
            });
        }
    }

    /// Finalize aggregate (compute derived values)
    pub fn finalize(&mut self) {
        // Compute average temperature from thermal energy
        if !self.material_volumes.is_empty() {
            // Simplified: assume uniform temperature
            self.avg_temperature = 293.15;
        }
    }

    /// Merge another aggregate into this one
    pub fn merge(&mut self, other: &LodAggregate) {
        for (mat, vol) in &other.material_volumes {
            *self.material_volumes.entry(*mat).or_insert(0.0) += vol;
        }
        
        self.total_thermal_energy += other.total_thermal_energy;
        
        for (proc, throughput) in &other.process_throughputs {
            *self.process_throughputs.entry(*proc).or_insert(0.0) += throughput;
        }
        
        self.power_consumption += other.power_consumption;
        self.heat_generation += other.heat_generation;
        
        for (entity, count) in &other.entity_counts {
            *self.entity_counts.entry(*entity).or_insert(0) += count;
        }
        
        self.logistics_flows.merge(&other.logistics_flows);
        
        if let Some(other_bounds) = other.bounds {
            self.bounds = Some(match self.bounds {
                Some(mut b) => {
                    b.merge(&other_bounds);
                    b
                },
                None => other_bounds,
            });
        }
    }

    /// Get dominant material
    pub fn dominant_material(&self) -> Option<MaterialId> {
        self.material_volumes
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| *k)
    }

    /// Total block count (approximate)
    pub fn estimated_block_count(&self) -> u64 {
        self.material_volumes.values().map(|v| (v * (REGION_BLOCK_SIZE as f32).powi(3)) as u64).sum()
    }
}

/// Aggregated logistics flows
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LogisticsAggregate {
    pub item_flow_rate: f32,      // items/tick
    pub fluid_flow_rate: f32,     // m³/tick
    pub gas_flow_rate: f32,       // m³/tick
    pub heat_flow_rate: f32,      // W
    pub power_flow_rate: f32,     // W
    pub data_flow_rate: f32,      // bits/tick
}

impl LogisticsAggregate {
    pub fn merge(&mut self, other: &LogisticsAggregate) {
        self.item_flow_rate += other.item_flow_rate;
        self.fluid_flow_rate += other.fluid_flow_rate;
        self.gas_flow_rate += other.gas_flow_rate;
        self.heat_flow_rate += other.heat_flow_rate;
        self.power_flow_rate += other.power_flow_rate;
        self.data_flow_rate += other.data_flow_rate;
    }
}

/// LOD Manager - handles transitions between detail levels
pub struct LodManager {
    /// Aggregates per region
    aggregates: HashMap<RegionKey, LodAggregate>,
    /// Update intervals (ticks)
    update_interval: u64,
    /// Last update per region
    last_update: HashMap<RegionKey, u64>,
    /// Distance thresholds for LOD levels
    lod_distances: [f32; 4], // LOD0, LOD1, LOD2, LOD3
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LodManager {
    pub fn new() -> Self {
        Self {
            aggregates: HashMap::new(),
            update_interval: 60, // Update every 60 ticks (1 second at 60Hz)
            last_update: HashMap::new(),
            lod_distances: [48.0, 96.0, 192.0, 384.0], // meters
        }
    }

    pub fn with_distances(d0: f32, d1: f32, d2: f32, d3: f32) -> Self {
        Self {
            lod_distances: [d0, d1, d2, d3],
            ..Default::default()
        }
    }

    /// Check if region should be updated
    pub fn should_update(&self, region: RegionKey, current_tick: u64) -> bool {
        let last = self.last_update.get(&region).copied().unwrap_or(0);
        current_tick - last >= self.update_interval
    }

    /// Update aggregate for region
    pub fn update(&mut self, region: RegionKey, aggregate: LodAggregate) {
        self.aggregates.insert(region, aggregate);
        // last_update will be set by caller
    }

    /// Invalidate region (force update next check)
    pub fn invalidate_region(&mut self, region: RegionKey) {
        self.last_update.remove(&region);
    }

    /// Get aggregate for region
    pub fn get(&self, region: RegionKey) -> Option<&LodAggregate> {
        self.aggregates.get(&region)
    }

    /// Get LOD level for distance
    pub fn lod_level_for_distance(&self, distance: f32) -> u8 {
        for (i, &d) in self.lod_distances.iter().enumerate() {
            if distance <= d {
                return i as u8;
            }
        }
        3 // Maximum LOD
    }

    /// Get distance threshold for LOD level
    pub fn distance_for_lod(&self, level: u8) -> f32 {
        self.lod_distances.get(level as usize).copied().unwrap_or(self.lod_distances[3])
    }

    /// Get all aggregates
    pub fn all_aggregates(&self) -> impl Iterator<Item = (&RegionKey, &LodAggregate)> {
        self.aggregates.iter()
    }

    /// Remove region
    pub fn remove_region(&mut self, region: RegionKey) {
        self.aggregates.remove(&region);
        self.last_update.remove(&region);
    }

    /// Memory usage
    pub fn memory_usage(&self) -> usize {
        self.aggregates.len() * std::mem::size_of::<LodAggregate>() + 
        self.last_update.len() * std::mem::size_of::<(RegionKey, u64)>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_aggregate() {
        let mut agg = LodAggregate::new(RegionKey::zero());
        assert!(agg.dominant_material().is_none());
        
        agg.material_volumes.insert(MaterialId(1), 100.0);
        agg.material_volumes.insert(MaterialId(2), 50.0);
        
        assert_eq!(agg.dominant_material(), Some(MaterialId(1)));
    }

    #[test]
    fn test_lod_manager() {
        let mut lod = LodManager::new();
        assert_eq!(lod.lod_level_for_distance(10.0), 0);
        assert_eq!(lod.lod_level_for_distance(50.0), 1);
        assert_eq!(lod.lod_level_for_distance(100.0), 2);
        assert_eq!(lod.lod_level_for_distance(500.0), 3);
    }

    #[test]
    fn test_lod_merge() {
        let mut agg1 = LodAggregate::new(RegionKey::zero());
        agg1.material_volumes.insert(MaterialId(1), 100.0);
        
        let mut agg2 = LodAggregate::new(RegionKey::zero());
        agg2.material_volumes.insert(MaterialId(1), 50.0);
        agg2.material_volumes.insert(MaterialId(2), 25.0);
        
        agg1.merge(&agg2);
        assert_eq!(agg1.material_volumes.get(&MaterialId(1)), Some(&150.0));
        assert_eq!(agg1.material_volumes.get(&MaterialId(2)), Some(&25.0));
    }
}