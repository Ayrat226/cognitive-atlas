/// Unit tests for the materials and crafting system
#[cfg(test)]
mod tests {
    use super::*;
    use lithos_engine_math::Fixed;
    
    #[test]
    fn test_elements_exist() {
        let materials = ThermodynamicCalculator::new();
        
        // Check that basic elements exist
        assert!(materials.get_material(MaterialId::HYDROGEN).is_some());
        assert!(materials.get_material(MaterialId::OXYGEN).is_some());
        assert!(materials.get_material(MaterialId::WATER).is_some());
        
        let oxygen = materials.get_material(MaterialId::OXYGEN).unwrap();
        assert_eq!(oxygen.atomic_number, 8);
        assert!(oxygen.is_element);
    }
    
    #[test]
    fn test_alloy_creation() {
        let materials = ThermodynamicCalculator::new();
        let copper = materials.get_material(MaterialId::COPPER).unwrap();
        let tin = materials.get_material(MaterialId::TIN).unwrap();
        
        // Create bronze alloy
        let bronze_components = vec![
            (MaterialId::COPPER, Fixed::from_f32(0.88)),
            (MaterialId::TIN, Fixed::from_f32(0.12)),
        ];
        
        let bronze_result = materials.calculate_alloy_phase(&bronze_components, Fixed::from_f32(1150.0 + 273.15));
        assert!(bronze_result.is_ok());
        
        let (phase, structure, _) = bronze_result.unwrap();
        assert_eq!(phase, MaterialPhase::Solid);
        assert_eq!(structure, CrystalStructure::Cubic);
    }
    
    #[test]
    fn test_crafting_quality() {
        let mut crafting = CraftingManager::new();
        
        // Mock player tools with a stone pickaxe (basic tool)
        let stone_pickaxe = ItemId(410);
        crafting.add_tool(Tool::new(
            stone_pickaxe,
            "Stone Pickaxe".to_string(),
            MaterialId::STONE,
            Quality::Normal,
        ));
        
        // Test crafting bronze with basic tools
        let result = crafting.craft_recipe("make_bronze", &[stone_pickaxe]);
        assert!(result.success);
        assert_eq!(result.outputs[0].2, Quality::Good); // Should be Good quality
        
        // Test free-form alloy experimentation
        let freeform_result = crafting.craft_freeform(
            "metalworking",
            &[
                (MaterialId::COPPER, Fixed::from_f32(0.9)),
                (MaterialId::TIN, Fixed::from_f32(0.1)),
            ],
            Fixed::from_f32(1150.0 + 273.15), // Hot enough for bronze
            Fixed::from_f32(30.0), // Process time
            &[stone_pickaxe],
            Fixed::from_f32(1.0), // Perfect stirring
        );
        
        assert!(freeform_result.success);
        assert_eq!(freeform_result.outputs[0].2, Quality::Excellent); // Should be Excellent with good conditions
    }
    
    #[test]
    fn test_material_properties() {
        let materials = ThermodynamicCalculator::new();
        
        let copper = materials.get_material(MaterialId::COPPER).unwrap();
        let bronze = materials.alloy(
            MaterialId::BRONZE,
            vec![
                (MaterialId::COPPER, Fixed::from_f32(0.88)),
                (MaterialId::TIN, Fixed::from_f32(0.12)),
            ],
            Fixed::from_f32(8.8), // Approximate density of bronze
            Fixed::from_f32(0.4), // Thermal conductivity of bronze
            Fixed::from_f32(0.1), // Electrical conductivity of bronze
            Fixed::from_f32(100e9), // Young's modulus of bronze
            Fixed::from_f32(300e6), // Yield strength of bronze
            (0.9, 0.7, 0.8), // Yellow color
        );
        
        let diffuse = bronze.youngs_modulus.to_f32();
        let strength = bronze.yield_strength.to_f32();
        
        println!("Bronze Young's modulus: {} GPa", diffuse / 1e9);
        println!("Bronze yield strength: {} MPa", strength / 1e6);
        
        // Basic sanity checks
        assert!(diffuse > 90e9); // Should be close to the target value
        assert!(strength > 150e6); // Should be stronger than 150 MPa
    }
    
    #[test]
    fn test_phase_diagram() {
        let materials = ThermodynamicCalculator::new();
        
        // Test Cu-Sn phase diagram
        let diagram = materials.get_phase_diagram(MaterialId::COPPER, MaterialId::TIN).unwrap();
        assert_eq!(diagram.components[0].0, 5); // Copper ID is COPPER (from u32 const)
        assert_eq!(diagram.components[1].0, 6); // Tin ID is 6
        
        println!("Cu-Sn eutectic composition: {:?}", 
                 diagram.eutectic.composition_b.to_f32() * 100.0, "%");
        println!("Cu-Sn eutectic temperature: {:?} K", 
                 diagram.eutectic.temperature.to_f32());
    }
}