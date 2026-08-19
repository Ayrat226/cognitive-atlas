//! GPU-driven mesh rendering pipeline

use std::sync::Arc;
use ash::vk;
use lithos_engine_memory::GLOBAL_STATS;
use lithos_engine_profiler::Profiler;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Vulkan error: {0}")]
    Vk(#[from] ash::vk::Result),
    #[error("Buffer creation failed: {0}")]
    Buffer(String),
    #[error("Pipeline creation failed: {0}")]
    Pipeline(String),
}

/// Vertex structure for voxel meshes
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MeshVertex {
    pub position: [f32; 3],      // 12 bytes
    pub normal: [f32; 3],        // 12 bytes  
    pub uv: [f32; 2],            // 8 bytes
    pub material_id: u32,        // 4 bytes
    pub _padding: u32,           // 4 bytes (align to 32 bytes)
}

/// Instance data for instanced rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct InstanceData {
    pub model_matrix: [[f32; 4]; 4],  // 64 bytes
    pub material_id: u32,             // 4 bytes
    pub _padding: [u32; 3],           // 12 bytes
}

/// Draw command for indirect rendering
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DrawIndexedIndirectCommand {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
    pub first_instance: u32,
}

/// Mesh buffer set for a chunk
pub struct ChunkMeshBuffers {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub indirect_buffer: vk::Buffer,
    pub indirect_buffer_memory: vk::DeviceMemory,
    pub index_count: u32,
    pub vertex_count: u32,
    pub aabb: lithos_engine_math::AABB,
}

/// Mesh pipeline for voxel rendering
pub struct MeshPipeline {
    device: Arc<ash::Device>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    vertex_input_binding: vk::VertexInputBindingDescription,
    vertex_input_attributes: [vk::VertexInputAttributeDescription; 4],
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl MeshPipeline {
    pub fn new(device: Arc<ash::Device>, descriptor_pool: vk::DescriptorPool) -> Result<Self, MeshError> {
        let vertex_input_binding = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<MeshVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let vertex_input_attributes = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),                              // position
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12),                             // normal
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(24),                             // uv
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32_UINT)
                .offset(32),                             // material_id
        ];

        // Create descriptor set layout
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&binding));

        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;

        // Create pipeline layout
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(128);  // Model matrix + material params

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        // Create graphics pipeline (will be fully configured later with shaders)
        let pipeline = vk::Pipeline::null();

        // Allocate descriptor sets
        let set_layouts = vec![descriptor_set_layout; 3]; // frames in flight
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&set_layouts);

        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_info) }?;

        Ok(Self {
            device,
            pipeline_layout,
            pipeline,
            vertex_input_binding,
            vertex_input_attributes,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_sets,
        })
    }

    pub fn create_chunk_buffers(
        device: &ash::Device,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        vertices: &[MeshVertex],
        indices: &[u32],
        commands: &[DrawIndexedIndirectCommand],
    ) -> Result<ChunkMeshBuffers, MeshError> {
        let vertex_size = (vertices.len() * std::mem::size_of::<MeshVertex>()) as vk::DeviceSize;
        let index_size = (indices.len() * std::mem::size_of::<u32>()) as vk::DeviceSize;
        let indirect_size = (commands.len() * std::mem::size_of::<DrawIndexedIndirectCommand>()) as vk::DeviceSize;

        // Create buffers
        let vertex_buffer_info = vk::BufferCreateInfo::default()
            .size(vertex_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let index_buffer_info = vk::BufferCreateInfo::default()
            .size(index_size)
            .usage(vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let indirect_buffer_info = vk::BufferCreateInfo::default()
            .size(indirect_size)
            .usage(vk::BufferUsageFlags::INDIRECT_BUFFER | vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let vertex_buffer = unsafe { device.create_buffer(&vertex_buffer_info, None) }?;
        let index_buffer = unsafe { device.create_buffer(&index_buffer_info, None) }?;
        let indirect_buffer = unsafe { device.create_buffer(&indirect_buffer_info, None) }?;

        // Allocate memory (simplified - would use proper allocator in practice)
        let vertex_buffer_memory = Self::allocate_buffer_memory(device, memory_properties, vertex_buffer, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
        let index_buffer_memory = Self::allocate_buffer_memory(device, memory_properties, index_buffer, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
        let indirect_buffer_memory = Self::allocate_buffer_memory(device, memory_properties, indirect_buffer, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;

        // Bind memory
        unsafe {
            device.bind_buffer_memory(vertex_buffer, vertex_buffer_memory, 0)?;
            device.bind_buffer_memory(index_buffer, index_buffer_memory, 0)?;
            device.bind_buffer_memory(indirect_buffer, indirect_buffer_memory, 0)?;
        }

        // Upload data (would use staging buffer in practice)
        // ... upload vertices, indices, commands ...

        Ok(ChunkMeshBuffers {
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            indirect_buffer,
            indirect_buffer_memory,
            index_count: indices.len() as u32,
            vertex_count: vertices.len() as u32,
            aabb: lithos_engine_math::AABB::empty(),
        })
    }

    fn allocate_buffer_memory(
        device: &ash::Device,
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        buffer: vk::Buffer,
        flags: vk::MemoryPropertyFlags,
    ) -> Result<vk::DeviceMemory, MeshError> {
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
        let memory_type_index = Self::find_memory_type(memory_properties, requirements.memory_type_bits, flags)?;
        
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(requirements.size)
            .memory_type_index(memory_type_index);

        let memory = unsafe { device.allocate_memory(&alloc_info, None) }?;
        Ok(memory)
    }

    fn find_memory_type(
        memory_properties: &vk::PhysicalDeviceMemoryProperties,
        type_bits: u32,
        flags: vk::MemoryPropertyFlags,
    ) -> Result<u32, MeshError> {
        for i in 0..memory_properties.memory_type_count {
            if (type_bits & (1 << i)) != 0 && 
               (memory_properties.memory_types[i as usize].property_flags & flags) == flags {
                return Ok(i);
            }
        }
        Err(MeshError::Buffer("No suitable memory type found".to_string()))
    }

    pub fn bind_and_draw(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline: vk::Pipeline,
        vertex_buffer: vk::Buffer,
        index_buffer: vk::Buffer,
        indirect_buffer: vk::Buffer,
        draw_count: u32,
    ) {
        unsafe {
            self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline);
            self.device.cmd_bind_vertex_buffers(command_buffer, 0, std::slice::from_ref(&vertex_buffer), &[0]);
            self.device.cmd_bind_index_buffer(command_buffer, index_buffer, 0, vk::IndexType::UINT32);
            self.device.cmd_draw_indexed_indirect(command_buffer, indirect_buffer, 0, draw_count, std::mem::size_of::<DrawIndexedIndirectCommand>() as u32);
        }
    }
}

impl Drop for MeshPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}

/// GPU-driven culling compute shader data
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CullingConstants {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _pad1: f32,
    pub frustum_planes: [[f32; 4]; 6],
    pub lod_distances: [f32; 4],
    pub chunk_count: u32,
    pub _pad2: [u32; 3],
}

/// Frustum culling compute pipeline
pub struct CullingPipeline {
    device: Arc<ash::Device>,
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
}

impl CullingPipeline {
    pub fn new(device: Arc<ash::Device>) -> Result<Self, MeshError> {
        // Descriptor set layout for culling
        let bindings = [
            vk::DescriptorSetLayoutBinding::default()  // Indirect draw commands (storage buffer, read)
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()  // Output indirect commands (storage buffer, write)
                .binding(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()  // Culling constants (uniform buffer)
                .binding(2)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
            vk::DescriptorSetLayoutBinding::default()  // AABB buffer (storage buffer, read)
                .binding(3)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

        let descriptor_set_layout = unsafe { device.create_descriptor_set_layout(&layout_info, None) }?;

        // Pipeline layout
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::COMPUTE)
            .offset(0)
            .size(std::mem::size_of::<CullingConstants>() as u32);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        // Pipeline will be created with shader module
        let pipeline = vk::Pipeline::null();

        Ok(Self {
            device,
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
        })
    }

    pub fn dispatch_culling(
        &self,
        command_buffer: vk::CommandBuffer,
        pipeline: vk::Pipeline,
        descriptor_set: vk::DescriptorSet,
        constants: &CullingConstants,
        workgroup_count: u32,
    ) {
        unsafe {
            self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::COMPUTE, pipeline);
            self.device.cmd_bind_descriptor_sets(command_buffer, vk::PipelineBindPoint::COMPUTE, self.pipeline_layout, 0, std::slice::from_ref(&descriptor_set), &[]);
            self.device.cmd_push_constants(command_buffer, self.pipeline_layout, vk::ShaderStageFlags::COMPUTE, 0, bytemuck::cast_slice(std::slice::from_ref(constants)));
            self.device.cmd_dispatch(command_buffer, workgroup_count, 1, 1);
        }
    }
}

impl Drop for CullingPipeline {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        }
    }
}