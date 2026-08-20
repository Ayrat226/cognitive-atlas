//! Crafting system for LITHOS
//! Free-form crafting, tool effectiveness, quality based on precision

use std::collections::HashMap;
use lithos_engine_math::{Fixed, FixedVec3};

use crate::materials::{MaterialDef, MaterialId, MaterialInstance, ThermodynamicCalculator};

/// Unique item ID - what can be crafted or found in world
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub u32);

impl ItemId {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const DIRT: Self = Self(2);
    pub const COPPER_ORE: Self = Self(100);
    pub const TIN_ORE: Self = Self(101);
    pub const IRON_ORE: Self = Self(102);
    pub const COPPER_INGOT: Self = Self(200);
    pub const TIN_INGOT: Self = Self(201);
    pub const IRON_INGOT: Self = Self(202);
    pub const BRONZE_INGOT: Self = Self(203);
    pub const BRASS_INGOT: Self = Self(204);
    pub const STEEL_INGOT: Self = Self(205);
    pub const WOOD: Self = Self(300);
    pub const STICK: Self = Self(301);
    pub const STONE_AXE: Self = Self(400);
    pub const WOODEN_AXE: Self = Self(401);
    pub const COPPER_AXE: Self = Self(402);
    pub const IRON_AXE: Self = Self(403);
    pub const STEEL_AXE: Self = Self(404);
    pub const STONE_PICKAXE: Self = Self(410);
    pub const WOODEN_PICKAXE: Self = Self(411);
    pub const COPPER_PICKAXE: Self = Self(412);
    pub const IRON_PICKAXE: Self = Self(413);
    pub const STEEL_PICKAXE: Self = Self(414);
    pub const HAMMER: Self = Self(420);
    pub const ANVIL: Self = Self(500);
    pub const FURNACE: Self = Self(501);
    
    pub fn new(value: u32) -> Self {
        ItemId(value)
    }
    
    pub fn from_material(mat_id: MaterialId) -> Self {
        ItemId(mat_id.0 + 1000) // Сдвигаем material id
    }
}

/// Quality level of crafted item
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Quality {
    Poor,
    Normal,
    Good,
    Excellent,
    Masterwork,
}

impl Quality {
    pub fn multiplier(&self) -> Fixed {
        match self {
            Quality::Poor => Fixed::from_f32(0.5),
            Quality::Normal => Fixed::from_f32(1.0),
            Quality::Good => Fixed::from_f32(1.25),
            Quality::Excellent => Fixed::from_f32(1.5),
            Quality::Masterwork => Fixed::from_f32(2.0),
        }
    }
    
    pub fn from_score(score: Fixed) -> Self {
        let score_f32 = score.to_f32();
        if score_f32 >= 0.9 {
            Quality::Masterwork
        } else if score_f32 >= 0.75 {
            Quality::Excellent
        } else if score_f32 >= 0.6 {
            Quality::Good
        } else if score_f32 >= 0.4 {
            Quality::Normal
        } else {
            Quality::Poor
        }
    }
}

/// Crafting recipe - traditional fixed recipes
#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub inputs: Vec<(ItemId, u32)>, // (item_id, количество)
    pub outputs: Vec<(ItemId, u32)>, // (item_id, количество)
    pub station: CraftingStation,
    pub time: Fixed, // Время крафта в секундах
    pub required_temp: Option<Fixed>, // Требуемая температура станций
    pub base_quality: Quality, // Базовое качество
    pub skill_required: SkillType, // Требуемый навык
    pub tool_effectiveness: f32, // Насколько инструмент влияет на результат (0-1)
}

/// Free-form crafting parameters - allows player creativity
#[derive(Debug, Clone)]
pub struct FreeformRecipe {
    pub id: String,
    pub name: String,
    /// Allowed materials and their ranges
    pub allowed_inputs: Vec<(MaterialId, FixedRange)>, // (материал, диапазон масс)
    pub allowed_outputs: Vec<ItemId>, // Возможные результаты
    pub station: CraftingStation,
    pub min_time: Fixed, // Минимальное время обработки
    pub max_time: Fixed, // Максимальное время
    pub base_temp: Fixed, // Базовая температура для реакций
    pub quality_factors: QualityFactors, // Что влияет на качество
}

/// Range of allowed material amounts
#[derive(Debug, Clone, Copy)]
pub struct FixedRange {
    pub min: Fixed,
    pub max: Fixed,
    
    pub fn contains(&self, value: Fixed) -> bool {
        value >= self.min && value <= self.max
    }
    
    pub fn new(min: Fixed, max: Fixed) -> Self {
        Self { min, max }
    }
    
    pub fn clamp(&self, value: Fixed) -> Fixed {
        value.max(self.min).min(self.max)
    }
}

/// Factors that affect crafting quality
#[derive(Debug, Clone)]
pub struct QualityFactors {
    pub temperature_precision_weight: f32, // Точность температуры
    pub time_precision_weight: f32,       // Точность времени
    pub tool_quality_weight: f32,         // Качество инструмента
    pub skill_weight: f32,                // Уровень навыка
    pub purity_weight: f32,               // Чистота материалов
    pub stirring_weight: f32,             // Тщательность перемешивания
}

/// Crafting station type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftingStation {
    Hand,           // Ручное крафтить
    Campfire,       // Костёр
    Furnace,        // Печь
    Anvil,          // Наковальня
    Forge,          // Кузнечный горн
    Workbench,      // Верстак
    Loom,           // Ткацкий стан
    Smithy,         // Кузнечная мастерская
    AlchemyLab,     // Алхимическая лаборатория
    Smelter,        // Плавка
}

/// Type of skill affecting crafting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillType {
    Mining,
    Woodcutting,
    Smithing,
    Tailoring,
    Alchemy,
    Engineering,
    Farming,
    Cooking,
}

/// Crafting result
#[derive(Debug, Clone)]
pub struct CraftResult {
    pub success: bool,
    pub outputs: Vec<(ItemId, u32, Quality)>, // (item_id, количество, качество)
    pub byproducts: Vec<(ItemId, u32)>, // Побочные продукты
    pub experience_gained: Fixed, // Опыт навыка
    pub quality_score: Fixed, // Оценка качества (0-1)
    pub message: String, // Сообщение о результате
    pub consumed_fuel: Fixed, // Потреблённое топливо
}

/// Player skill level
#[derive(Debug, Clone)]
pub struct Skill {
    pub skill_type: SkillType,
    pub level: u32,           // 0-100
    pub experience: Fixed,    // Текущий опыт в уровне
    pub experience_to_next: Fixed, // Опыт до следующего уровня
}

impl Skill {
    pub fn new(skill_type: SkillType) -> Self {
        Self {
            skill_type,
            level: 0,
            experience: Fixed::ZERO,
            experience_to_next: Fixed::from_f32(100.0),
        }
    }
    
    pub fn add_experience(&mut self, amount: Fixed) -> bool {
        self.experience += amount;
        let leveled_up = self.experience >= self.experience_to_next;
        
        if leveled_up {
            self.level += 1;
            self.experience -= self.experience_to_next;
            self.experience_to_next = Fixed::from_f32(100.0) * 
                Fixed::from_f32(1.5_f32.powf(self.level as f32)); // Экспоненциальный рост
        }
        
        leveled_up
    }
    
    pub fn proficiency(&self) -> Fixed {
        // 0.0 to 1.0 based on level
        Fixed::from_f32((self.level as f32) / 100.0)
    }
}

/// Tool effectiveness based on material and quality
#[derive(Debug, Clone)]
pub struct Tool {
    pub id: ItemId,
    pub name: String,
    pub material: MaterialId,      // Из чего сделан инструмент
    pub quality: Quality,          // Качество изготовления
    pub durability: Fixed,         // Текущая прочность (0-1)
    pub max_durability: Fixed,     // Максимальная прочность
    pub effectiveness: Fixed,      // Базовая эффективность (0-1)
    pub special_effects: Vec<ToolEffect>, // Специфические эффекты
}

impl Tool {
    pub fn new(id: ItemId, name: String, material: MaterialId, quality: Quality) -> Self {
        let base_effectiveness = match material {
            MaterialId::WOOD => Fixed::from_f32(0.3),
            MaterialId::STONE => Fixed::from_f32(0.5),
            MaterialId::COPPER => Fixed::from_f32(0.7),
            MaterialId::IRON => Fixed::from_f32(0.8),
            MaterialId::STEEL => Fixed::from_f32(0.9),
            _ => Fixed::from_f32(0.1),
        };
        
        let quality_mod = quality.multiplier();
        
        Self {
            id,
            name,
            material,
            quality,
            durability: Fixed::ONE,
            max_durability: Fixed::ONE,
            effectiveness: base_effectiveness * quality_mod,
            special_effects: Vec::new(),
        }
    }
    
    pub fn use_tool(&mut self, amount: Fixed) -> bool {
        if self.durability <= amount {
            self.durability = Fixed::ZERO;
            false
        } else {
            self.durability -= amount;
            true
        }
    }
    
    pub fn repair(&mut self, amount: Fixed) {
        self.durability = (self.durability + amount).min(self.max_durability);
    }
    
    pub fn get_effective_multiplier(&self) -> Fixed {
        self.effectiveness * self.durability
    }
}

#[derive(Debug, Clone)]
pub enum ToolEffect {
    MiningSpeed(fixed::Fixed),    // Увеличение скорости добычи
    DurabilityBonus(fixed::Fixed), // Увеличение прочности
    PrecisionBonus(fixed::Fixed),  // Увеличение точности крафта
    HeatResistance(fixed::Fixed),  // Устойчивость к высоким температурам
    CorrosionResistance,           // Сопротивление коррозии
}

/// Crafting manager - handles both recipe-based and free-form crafting
pub struct CraftingManager {
    pub recipes: HashMap<String, Recipe>,
    pub freeform_recipes: HashMap<String, FreeformRecipe>,
    pub thermodynamic_calc: ThermodynamicCalculator,
    pub skills: HashMap<SkillType, Skill>,
    pub tools: HashMap<ItemId, Tool>,
}

impl CraftingManager {
    pub fn new() -> Self {
        let mut manager = Self {
            recipes: HashMap::new(),
            freeform_recipes: HashMap::new(),
            thermodynamic_calc: ThermodynamicCalculator::new(),
            skills: HashMap::new(),
            tools: HashMap::new(),
        };
        
        // Initialize basic skills
        for skill_type in [
            SkillType::Mining,
            SkillType::Woodcutting,
            SkillType::Smithing,
            SkillType::Tailoring,
            SkillType::Alchemy,
            SkillType::Engineering,
        ] {
            manager.skills.insert(skill_type, Skill::new(skill_type));
        }
        
        // Register basic recipes and freeform options
        manager.register_basic_recipes();
        manager.register_freeform_recipes();
        
        manager
    }
    
    fn register_basic_recipes(&mut self) {
        // Basic smelting recipes
        self.recipes.insert(
            "smelt_copper".to_string(),
            Recipe {
                id: "smelt_copper".to_string(),
                name: "Smelt Copper".to_string(),
                inputs: vec![(ItemId::COPPER_ORE, 1)],
                outputs: vec![(ItemId::COPPER_INGOT, 1)],
                station: CraftingStation::Furnace,
                time: Fixed::from_f32(10.0),
                required_temp: Some(Fixed::from_f32(1358.0 + 273.15)), // Медь плавится при 1085°C
                base_quality: Quality::Normal,
                skill_required: SkillType::Smithing,
                tool_effectiveness: 0.0,
            }
        );
        
        self.recipes.insert(
            "smelt_tin".to_string(),
            Recipe {
                id: "smelt_tin".to_string(),
                name: "Smelt Tin".to_string(),
                inputs: vec![(ItemId::TIN_ORE, 1)],
                outputs: vec![(ItemId::TIN_INGOT, 1)],
                station: CraftingStation::Furnace,
                time: Fixed::from_f32(8.0),
                required_temp: Some(Fixed::from_f32(232.0 + 273.15)), // Олово плавится при 232°C
                base_quality: Quality::Normal,
                skill_required: SkillType::Smithing,
                tool_effectiveness: 0.0,
            }
        );
        
        self.recipes.insert(
            "make_bronze".to_string(),
            Recipe {
                id: "make_bronze".to_string(),
                name: "Make Bronze".to_string(),
                inputs: vec![(ItemId::COPPER_INGOT, 9), (ItemId::TIN_INGOT, 1)],
                outputs: vec![(ItemId::BRONZE_INGOT, 1)],
                station: CraftingStation::Furnace,
                time: Fixed::from_f32(15.0),
                required_temp: Some(Fixed::from_f32(900.0 + 273.15)), // Сплав плавится ~900°C
                base_quality: Quality::Good,
                skill_required: SkillType::Smithing,
                tool_effectiveness: 0.3,
            }
        );
        
        self.recipes.insert(
            "make_steel".to_string(),
            Recipe {
                id: "make_steel".to_string(),
                name: "Make Steel".to_string(),
                inputs: vec![(ItemId::IRON_INGOT, 99), (ItemId::COPPER_INGOT, 1)], // Упрощенно: железо + углерод
                outputs: vec![(ItemId::STEEL_INGOT, 1)],
                station: CraftingStation::Furnace,
                time: Fixed::from_f32(20.0),
                required_temp: Some(Fixed::from_f32(1370.0 + 273.15)), // Сталь плавится ~1370°C
                base_quality: Quality::Excellent,
                skill_required: SkillType::Smithing,
                tool_effectiveness: 0.5,
            }
        );
        
        // Tool making recipes
        self.recipes.insert(
            "stone_axe".to_string(),
            Recipe {
                id: "stone_axe".to_string(),
                name: "Stone Axe".to_string(),
                inputs: vec![(ItemId::STONE, 3), (ItemId::STICK, 2)],
                outputs: vec![(ItemId::STONE_AXE, 1)],
                station: CraftingStation::Workbench,
                time: Fixed::from_f32(5.0),
                required_temp: None,
                base_quality: Quality::Normal,
                skill_required: SkillType::Woodcutting,
                tool_effectiveness: 0.2,
            }
        );
        
        self.recipes.insert(
            "iron_axe".to_string(),
            Recipe {
                id: "iron_axe".to_string(),
                name: "Iron Axe".to_string(),
                inputs: vec![(ItemId::IRON_INGOT, 3), (ItemId::STICK, 2)],
                outputs: vec![(ItemId::IRON_AXE, 1)],
                station: CraftingStation::Anvil,
                time: Fixed::from_f32(8.0),
                required_temp: Some(Fixed::from_f32(500.0 + 273.15)), // Для ковки
                base_quality: Quality::Good,
                skill_required: SkillType::Smithing,
                tool_effectiveness: 0.5,
            }
        );
    }
    
    fn register_freeform_recipes(&mut self) {
        // Free-form metalworking - allows experimenting with alloys
        self.freeform_recipes.insert(
            "metalworking".to_string(),
            FreeformRecipe {
                id: "metalworking".to_string(),
                name: "Metalworking".to_string(),
                allowed_inputs: vec![
                    (MaterialId::COPPER, FixedRange::new(Fixed::ZERO, Fixed::from_f32(1.0))),
                    (MaterialId::TIN, FixedRange::new(Fixed::ZERO, Fixed::from_f32(1.0))),
                    (MaterialId::IRON, FixedRange::new(Fixed::ZERO, Fixed::from_f32(1.0))),
                    (MaterialId::CARBON, FixedRange::new(Fixed::ZERO, Fixed::from_f32(0.1))), // Углерод как легирующий элемент
                ],
                allowed_outputs: vec![
                    ItemId::COPPER_INGOT,
                    ItemId::TIN_INGOT,
                    ItemId::IRON_INGOT,
                    ItemId::BRONZE_INGOT,
                    ItemId::BRASS_INGOT,
                    ItemId::STEEL_INGOT,
                ],
                station: CraftingStation::Forge,
                min_time: Fixed::from_f32(5.0),
                max_time: Fixed::from_f32(60.0),
                base_temp: Fixed::from_f32(800.0 + 273.15), // Базовая температура кузницы
                quality_factors: QualityFactors {
                    temperature_precision_weight: 0.3,
                    time_precision_weight: 0.2,
                    tool_quality_weight: 0.2,
                    skill_weight: 0.2,
                    purity_weight: 0.05,
                    stirring_weight: 0.05,
                },
            }
        );
        
        // Free-form woodworking
        self.freeform_recipes.insert(
            "woodworking".to_string(),
            FreeformRecipe {
                id: "woodworking".to_string(),
                name: "Woodworking".to_string(),
                allowed_inputs: vec![
                    (MaterialId::HYDROGEN, FixedRange::new(Fixed::ZERO, Fixed::from_f32(0.1))), // Влага в дереве
                    (MaterialId::CARBON, FixedRange::new(Fixed::from_f32(0.4), Fixed::from_f32(0.5))), // Основной компонент дерева
                    (MaterialId::OXYGEN, FixedRange::new(Fixed::from_f32(0.4), Fixed::from_f32(0.5))),
                ],
                allowed_outputs: vec![
                    ItemId::WOOD,
                    ItemId::STICK,
                    ItemId::WOODEN_AXE,
                    ItemId::STONE_AXE, // Для привязки камня к дереву
                ],
                station: CraftingStation::Workbench,
                min_time: Fixed::from_f32(2.0),
                max_time: Fixed::from_fib20.0),
                base_temp: Fixed::from_f32(298.15), // Комнатная температура
                quality_factors: QualityFactors {
                    temperature_precision_weight: 0.1,
                    time_precision_weight: 0.3,
                    tool_quality_weight: 0.3,
                    skill_weight: 0.2,
                    purity_weight: 0.05,
                    stirring_weight: 0.05,
                },
            }
        );
    }
    
    /// Attempt to craft using a traditional recipe
    pub fn craft_recipe(&mut self, recipe_id: &str, player_tools: &[ItemId]) -> CraftResult {
        let recipe = match self.recipes.get(recipe_id) {
            Some(r) => r,
            None => return CraftResult {
                success: false,
                outputs: Vec::new(),
                byproducts: Vec::new(),
                experience_gained: Fixed::ZERO,
                quality_score: Fixed::ZERO,
                message: "Recipe not found".to_string(),
                consumed_fuel: Fixed::ZERO,
            },
        };
        
        // Check if player has required tools
        let has_required_tool = player_tools.iter().any(|&tool_id| {
            self.tools.get(&tool_id).map_or(false, |tool| {
                tool.material != MaterialId::AIR // Простая проверка
            })
        });
        
        if !has_required_tool && recipe.tool_effectiveness > 0.0 {
            return CraftResult {
                success: false,
                outputs: Vec::new(),
                byproducts: Vec::new(),
                experience_gained: Fixed::ZERO,
                quality_score: Fixed::ZERO,
                message: "Required tool not available".to_string(),
                consumed_fuel: Fixed::ZERO,
            };
        }
        
        // Calculate base quality from skill
        let skill = self.skills.get(&recipe.skill_required)
            .unwrap_or(&Skill::new(recipe.skill_required));
        let skill_proficiency = skill.proficiency();
        
        // Tool effectiveness modifier
        let tool_bonus = if recipe.tool_effectiveness > 0.0 {
            let best_tool = player_tools.iter()
                .filter_map(|&tool_id| self.tools.get(&tool_id))
                .max_by(|a, b| a.get_effective_multiplier().partial_cmp(&b.get_effective_multiplier()).unwrap())
                .map(|tool| tool.get_effective_multiplier())
                .unwrap_or(Fixed::ZERO);
            
            Fixed::from_f32(recipe.tool_effectiveness) * tool_bonus
        } else {
            Fixed::ZERO
        };
        
        // Base quality calculation
        let base_quality_score = recipe.base_quality.multiplier() * 
            (Fixed::ONE + skill_proficiency * Fixed::from_f32(0.5) + tool_bonus);
        
        // Add some randomness for realism (in real implementation would use seeded RNG)
        let quality_variation = Fixed::from_f32(0.1); // ±10% вариация
        let final_quality_score = (base_quality_score * Fixed::from_f32(0.9)) + 
            (Fixed::from_f32(0.1) * Fixed::from_f32(0.5)); // Упрощенно
        
        let quality = Quality::from_score(final_quality_score.clamp(Fixed::ZERO, Fixed::ONE));
        
        // Gain experience
        let experience_gained = Fixed::from_f32(recipe.time.to_f32() * 0.1) * 
            skill_proficiency * quality.multiplier();
        
        // Update skill
        if let Some(skill) = self.skills.get_mut(&recipe.skill_required) {
            skill.add_experience(experience_gained);
        }
        
        CraftResult {
            success: true,
            outputs: recipe.outputs.iter()
                .map(|&(item_id, count)| (item_id, count, quality))
                .collect(),
            byproducts: Vec::new(),
            experience_gained,
            quality_score: final_quality_score,
            message: format!("Crafted {}x{}", recipe.outputs[0].1, recipe.name),
            consumed_fuel: if recipe.station == CraftingStation::Furnace {
                Fixed::from_f32(recipe.time.to_f32() * 0.1) // Топливо пропорционально времени
            } else {
                Fixed::ZERO
            },
        }
    }
    
    /// Attempt free-form crafting - player experiments with materials
    pub fn craft_freeform(&mut self, 
                         recipe_id: &str,
                         inputs: &[(MaterialId, Fixed)], // (материал, масса в кг)
                         station_temp: Fixed,
                         process_time: Fixed,
                         player_tools: &[ItemId],
                         stirring_quality: Fixed) -> CraftResult {
        let recipe = match self.freeform_recipes.get(recipe_id) {
            Some(r) => r,
            None => return CraftResult {
                success: false,
                outputs: Vec::new(),
                byproducts: Vec::new(),
                experience_gained: Fixed::ZERO,
                quality_score: Fixed::ZERO,
                message: "Freeform recipe not found".to_string(),
                consumed_fuel: Fixed::ZERO,
            },
        };
        
        // Validate inputs against allowed materials
        let mut validated_inputs = Vec::new();
        let mut total_mass = Fixed::ZERO;
        
        for &(mat_id, mass) in inputs {
            // Check if material is allowed
            let is_allowed = recipe.allowed_inputs.iter()
                .any(|&(allowed_id, range)| 
                    allowed_id == mat_id && range.contains(mass));
            
            if !is_allowed {
                return CraftResult {
                    success: false,
                    outputs: Vec::new(),
                    byproducts: Vec::new(),
                    experience_gained: Fixed::ZERO,
                    quality_score: Fixed::ZERO,
                    message: format!("Material {} not allowed in this recipe", mat_id.0),
                    consumed_fuel: Fixed::ZERO,
                };
            }
            
            validated_inputs.push((mat_id, mass));
            total_mass += mass;
        }
        
        // Validate time
        if process_time < recipe.min_time || process_time > recipe.max_time {
            return CraftResult {
                success: false,
                outputs: Vec::new(),
                byproducts: Vec::new(),
                experience_gained: Fixed::ZERO,
                quality_score: Fixed::ZERO,
                message: "Process time outside allowed range".to_string(),
                consumed_fuel: Fixed::ZERO,
            };
        }
        
        // Validate temperature
        let temp_diff = (station_temp - recipe.base_temp).abs();
        let temp_tolerance = Fixed::from_f32(50.0); // ±50K допуск
        if temp_diff > temp_tolerance {
            return CraftResult {
                success: false,
                outputs: Vec::new(),
                byproducts: Vec::new(),
                experience_gained: Fixed::ZERO,
                quality_score: Fixed::ZERO,
                message: "Temperature outside optimal range".to_string(),
                consumed_fuel: Fixed::ZERO,
            };
        }
        
        // Calculate quality based on precision
        let temp_quality = (Fixed::ONE - temp_diff / temp_tolerance.min(Fixed::ONE))
            .max(Fixed::ZERO) * Fixed::from_f32(recipe.quality_factors.temperature_precision_weight);
        
        let time_quality = if process_time > recipe.min_time && process_time < recipe.max_time {
            let time_mid = (recipe.min_time + recipe.max_time) / Fixed::from_f32(2.0);
            let time_diff = (process_time - time_mid).abs();
            let time_tolerance = (recipe.max_time - recipe.min_time) / Fixed::from_f32(2.0);
            (Fixed::ONE - time_diff / time_tolerance.min(Fixed::ONE))
                .max(Fixed::ZERO) * Fixed::from_f32(recipe.quality_factors.time_precision_weight)
        } else {
            Fixed::ZERO
        };
        
        // Tool quality
        let tool_quality = if recipe.quality_factors.tool_quality_weight > 0.0 {
            let best_tool = player_tools.iter()
                .filter_map(|&tool_id| self.tools.get(&tool_id))
                .max_by(|a, b| a.get_effective_multiplier().partial_cmp(&b.get_effective_multiplier()).unwrap())
                .map(|tool| tool.quality.multiplier())
                .unwrap_or(Fixed::from_f32(0.5));
            
            Fixed::from_f32(recipe.quality_factors.tool_quality_weight) * tool_quality
        } else {
            Fixed::ZERO
        };
        
        // Skill contribution
        let skill_quality = if recipe.quality_factors.skill_weight > 0.0 {
            // Используем средний навык для всех требуемых навыков (упрощение)
            let avg_skill = self.skills.values()
                .map(|skill| skill.proficiency())
                .reduce(|a, b| a + b)
                .unwrap_or(Fixed::ZERO) / 
                Fixed::from_f32(self.skills.len() as f32);
            
            Fixed::from_f32(recipe.quality_factors.skill_weight) * avg_skill
        } else {
            Fixed::ZERO
        };
        
        // Purity - check if inputs match ideal ratios (simplified)
        let purity_quality = Fixed::from_f32(recipe.quality_factors.purity_weight) * 
            Fixed::from_f32(0.8); // Упрощенно предполагаем 80% чистоты
        
        // Stirring quality
        let stirring_quality_val = stirring_quality * 
            Fixed::from_f32(recipe.quality_factors.stirring_weight);
        
        // Total quality score
        let mut quality_score = temp_quality + time_quality + tool_quality + 
            skill_quality + purity_quality + stirring_quality_val;
        
        quality_score = quality_score.min(Fixed::ONE).max(Fixed::ZERO);
        
        let quality = Quality::from_score(quality_score);
        
        // Determine output based on composition and thermodynamics
        let (output_item, output_count, byproducts) = self.determine_freeform_output(
            &validated_inputs,
            station_temp,
            process_time,
            &recipe,
        )?;
        
        // Experience gained
        let experience_gained = Fixed::from_f32(process_time.to_f32() * 0.05) * 
            quality_score * Fixed::from_f32(2.0); // Больше опыта за эксперименты
        
        // Update relevant skills (simplified - just smithing for metalworking)
        if let Some(skill) = self.skills.get_mut(&SkillType::Smithing) {
            skill.add_experience(experience_gained);
        }
        
        // Fuel consumption
        let consumed_fuel = if recipe.station == CraftingStation::Furnace || 
                               recipe.station == CraftingStation::Forge {
            Fixed::from_f32(process_time.to_f32() * 0.15) // Больше топлива для нагрева
        } else {
            Fixed::ZERO
        };
        
        CraftResult {
            success: true,
            outputs: vec![(output_item, output_count, quality)],
            byproducts,
            experience_gained,
            quality_score,
            message: format!("Successfully crafted {}x{}", output_count, 
                           self.get_item_name(&output_item)),
            consumed_fuel,
        }
    }
    
    fn determine_freeform_output(&self,
                                inputs: &[(MaterialId, Fixed)],
                                temperature: Fixed,
                                time: Fixed,
                                recipe: &FreeformRecipe) -> Result<(ItemId, u32, Vec<(ItemId, u32)>), String> {
        // Simplified logic: determine output based on dominant materials and temperature
        let total_mass: Fixed = inputs.iter().map(|&(_, mass)| mass).sum();
        
        if total_mass <= Fixed::ZERO {
            return Err("No material provided".to_string());
        }
        
        // Calculate mass fractions
        let mut fractions = Vec::new();
        for &(mat_id, mass) in inputs {
            let fraction = if total_mass > Fixed::ZERO { mass / total_mass } else { Fixed::ZERO };
            fractions.push((mat_id, fraction));
        }
        
        // Find dominant material
        let mut dominant_mat = MaterialId::AIR;
        let mut max_fraction = Fixed::ZERO;
        
        for &(mat_id, fraction) in &fractions {
            if fraction > max_fraction {
                max_fraction = fraction;
                dominant_mat = mat_id;
            }
        }
        
        // Determine output based on combinations and temperature
        let output = if dominant_mat == MaterialId::COPPER {
            // Check for tin to make bronze
            let tin_fraction = fractions.iter()
                .find(|&&(mat_id, _)| mat_id == MaterialId::TIN)
                .map(|&(_, frac)| frac)
                .unwrap_or(Fixed::ZERO);
            
            if tin_fraction > Fixed::from_f32(0.1) && temperature > Fixed::from_f32(900.0 + 273.15) {
                // Enough tin and hot enough for bronze
                (ItemId::BRONZE_INGOT, 1, Vec::new())
            } else if temperature > Fixed::from_f32(1085.0 + 273.15) {
                // Just copper melting
                (ItemId::COPPER_INGOT, 1, Vec::new())
            } else {
                // Too cold - no reaction
                (ItemId::COPPER_INGOT, 0, vec![(ItemId::COPPER_ORE, 1)]) // Return as ore
            }
        } else if dominant_mat == MaterialId::TIN {
            if temperature > Fixed::from_f32(232.0 + 273.15) {
                (ItemId::TIN_INGOT, 1, Vec::new())
            } else {
                (ItemId::TIN_INGOT, 0, vec![(ItemId::TIN_ORE, 1)])
            }
        } else if dominant_mat == MaterialId::IRON {
            // Check for carbon to make steel
            let carbon_fraction = fractions.iter()
                .find(|&&(mat_id, _)| mat_id == MaterialId::CARBON)
                .map(|&(_, frac)| frac)
                .unwrap_or(Fixed::ZERO);
            
            if carbon_fraction > Fixed::from_f3ed(0.01) && temperature > Fixed::from_f32(1370.0 + 273.15) {
                // Enough carbon and hot enough for steel
                (ItemId::STEEL_INGOT, 1, Vec::new())
            } else if temperature > Fixed::from_f32(1538.0 + 273.15) {
                // Just iron melting
                (ItemId::IRON_INGOT, 1, Vec::new())
            } else {
                (ItemId::IRON_INGOT, 0, vec![(ItemId::IRON_ORE, 1)])
            }
        } else if dominant_mat == MaterialId::HYDROGEN && 
                  fractions.iter().any(|&(mat_id, _)| mat_id == MaterialId::OXYGEN) {
            // Water formation
            let o2_fraction = fractions.iter()
                .find(|&&(mat_id, _)| mat_id == MaterialId::OXYGEN)
                .map(|&(_, frac)| frac)
                .unwrap_or(Fixed::ZERO);
            
            if o2_fraction > Fixed::from_f32(0.33) && temperature > Fixed::from_f32(373.15) {
                (ItemId::AIR, 1, vec![(ItemId::WATER, 1)]) // Produce water vapor/steam
            } else {
                (ItemId::AIR, 0, vec![(ItemId::HYDROGEN, 1), (ItemId::OXYGEN, 1)]) // Unreacted
            }
        } else {
            // Default - try to convert to basic item
            (ItemId::from_material(dominant_mat), 1, Vec::new())
        };
        
        Ok((output.0, output.1, output.2))
    }
    
    fn get_item_name(&self, item_id: &ItemId) -> String {
        match item_id.0 {
            0 => "Air".to_string(),
            1 => "Stone".to_string(),
            2 => "Dirt".to_string(),
            100 => "Copper Ore".to_string(),
            101 => "Tin Ore".to_string(),
            102 => "Iron Ore".to_string(),
            200 => "Copper Ingot".to_string(),
            201 => "Tin Ingot".to_string(),
            202 => "Iron Ingot".to_string(),
            203 => "Bronze Ingot".to_string(),
            204 => "Brass Ingot".to_string(),
            205 => "Steel Ingot".to_string(),
            300 => "Wood".to_string(),
            301 => "Stick".to_string(),
            400 => "Stone Axe".to_string(),
            401 => "Wooden Axe".to_string(),
            402 => "Copper Axe".to_string(),
            403 => "Iron Axe".to_string(),
            404 => "Steel Axe".to_string(),
            410 => "Stone Pickaxe".to_string(),
            411 => "Wooden Pickaxe".to_string(),
            412 => "Copper Pickaxe".to_string(),
            413 => "Iron Pickaxe".to_string(),
            414 => "Steel Pickaxe".to_string(),
            420 => "Hammer".to_string(),
            500 => "Anvil".to_string(),
            501 => "Furnace".to_string(),
            _ => format!("Item {}", item_id.0),
        }
    }
    
    /// Get current skill level
    pub fn get_skill_level(&self, skill_type: SkillType) -> u32 {
        self.skills.get(&skill_type).map(|s| s.level).unwrap_or(0)
    }
    
    /// Get tool by ID
    pub fn get_tool(&self, tool_id: ItemId) -> Option<&Tool> {
        self.tools.get(&tool_id)
    }
    
    /// Add or update a tool
    pub fn add_tool(&mut self, tool: Tool) {
        self.tools.insert(tool.id, tool);
    }
}

// Helper module for fixed point operations (simplified)
mod fixed {
    use super::Fixed;
    
    pub trait FixedExt {
        fn sqr(self) -> Self;
        fn powf(self, exp: f32) -> Self;
    }
    
    impl FixedExt for Fixed {
        fn sqr(self) -> Self {
            self * self
        }
        
        fn powf(self, exp: f32) -> Self {
            // Simplified implementation - in real code would use proper fixed-point pow
            Fixed::from_f32(self.to_f32().powf(exp))
        }
    }
}