use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    decimals::round_to_2,
    equipment::Blueprint,
    heroes::{create_sim_hero, SimHero},
    inputs::{create_hero_input, HeroInput},
    skills::{HeroSkill, InnateSkill},
};

/// Defines a HeroClass that contains info on base stats, allowed equipment, etc.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HeroClass {
    class: String,
    prerequisite: String,
    gold_hire_cost: u32,
    gem_hire_cost: u32,

    base_hp: Vec<f64>,
    base_atk: Vec<f64>,
    base_def: Vec<f64>,
    base_eva: f64,
    base_crit_chance: f64,
    base_crit_mult: f64,
    base_threat_rating: u16,

    element_type: String,
    equipment_allowed: [Vec<String>; 6],

    innate_skills: [String; 4],
}

pub fn _create_hero_class(
    class: String,
    prerequisite: String,
    gold_hire_cost: u32,
    gem_hire_cost: u32,

    base_hp: Vec<f64>,
    base_atk: Vec<f64>,
    base_def: Vec<f64>,
    base_eva: f64,
    base_crit_chance: f64,
    base_crit_mult: f64,
    base_threat_rating: u16,

    element_type: String,
    equipment_allowed: [Vec<String>; 6],

    innate_skills: [String; 4],
) -> HeroClass {
    return HeroClass {
        class,
        prerequisite,
        gold_hire_cost,
        gem_hire_cost,

        base_hp,
        base_atk,
        base_def,
        base_eva,
        base_crit_chance,
        base_crit_mult,
        base_threat_rating,

        element_type,
        equipment_allowed,

        innate_skills,
    };
}

/// Defines a Hero that contains info on base stats, equipment, and skills
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Hero {
    identifier: String,
    class: String,
    level: u8,
    rank: u8,
    innate_tier: u8,

    hp: f64,
    atk: f64,
    def: f64,
    eva: f64,
    crit_chance: f64,
    crit_mult: f64,
    threat_rating: u16,
    element_type: String,

    atk_modifier: f64,
    def_modifier: f64,

    hp_seeds: u8,
    atk_seeds: u8,
    def_seeds: u8,

    skills: [String; 4],

    equipment_equipped: [String; 6],
    equipment_quality: [String; 6],
    elements_socketed: [String; 6],
    spirits_socketed: [String; 6],
}

pub fn create_hero(
    identifier: String,
    class: String,
    level: u8,
    rank: u8,
    innate_tier: u8,

    hp: f64,
    atk: f64,
    def: f64,
    eva: f64,
    crit_chance: f64,
    crit_mult: f64,
    threat_rating: u16,
    element_type: String,

    atk_modifier: f64,
    def_modifier: f64,

    hp_seeds: u8,
    atk_seeds: u8,
    def_seeds: u8,

    skills: [String; 4],

    equipment_equipped: [String; 6],
    equipment_quality: [String; 6],
    elements_socketed: [String; 6],
    spirits_socketed: [String; 6],
) -> Hero {
    return Hero {
        identifier,
        class,
        level,
        rank,
        innate_tier,

        hp,
        atk,
        def,
        eva,
        crit_chance,
        crit_mult,
        threat_rating,
        element_type,

        atk_modifier,
        def_modifier,

        hp_seeds,
        atk_seeds,
        def_seeds,

        skills,

        equipment_equipped,
        equipment_quality,
        elements_socketed,
        spirits_socketed,
    };
}

impl Hero {
    pub fn validate_equipment(
        &self,
        bp_map: &HashMap<String, Blueprint>,
        hero_classes: &HashMap<String, HeroClass>,
    ) {
        if !hero_classes.contains_key(&self.class) {
            panic!(
                "Encountered unknown class {} for hero {}",
                self.class, self.identifier
            );
        }
        let class = hero_classes.get(&self.class).unwrap();

        for (i, equipment) in self.equipment_equipped.iter().enumerate() {
            if !bp_map.contains_key(equipment) {
                panic!(
                    "Equipment {} could not be validated as a known item",
                    equipment
                );
            }
            let blueprint = bp_map.get(equipment).unwrap();
            if !class.equipment_allowed[i].contains(&blueprint.get_type()) {
                panic!(
                    "Equipment {} is of type {} that is not allowed for this class in this slot (# {}). Valid options: {:#?}",
                    equipment,
                    blueprint.get_type(),
                    i,
                    class.equipment_allowed,
                )
            }
        }
    }

    pub fn calculate_innate_skill_name(
        &self,
        class_innate_skill_names_map: &HashMap<String, String>,
    ) -> String {
        if !class_innate_skill_names_map.contains_key(&self.class) {
            // Class not found in map
            panic!(
                "Class {} could not be found in keys for class_innate_skill_names_map",
                self.class
            );
        }

        let innate_skill = class_innate_skill_names_map[&self.class].clone();
        return innate_skill;
    }

    pub fn calculate_innate_tier(
        &mut self,
        class_innate_skill_names_map: &HashMap<String, String>,
        innate_skill_map: &HashMap<String, InnateSkill>,
    ) {
        let element_qty = self.calculate_element_qty();
        let innate_skill = self.calculate_innate_skill_name(class_innate_skill_names_map);

        let mut innate_skill_variants: Vec<&InnateSkill> = innate_skill_map
            .values()
            .filter(|is| {
                is.get_tier_1_name() == innate_skill && is.get_element_qty_req() < element_qty
            })
            .collect::<Vec<&InnateSkill>>();

        innate_skill_variants.sort_unstable_by_key(|is| is.get_skill_tier());

        println!("Innate_Skill_Variants: {:#?}", innate_skill_variants);

        let innate_skill_info = innate_skill_variants[innate_skill_variants.len() - 1];

        self.innate_tier = innate_skill_info.get_skill_tier();
    }

    pub fn calculate_element_qty(&self) -> u16 {
        let mut element_qty = 0u16;
        for element_string in &self.elements_socketed {
            let split_vec: Vec<&str> = element_string.split(" ").collect();
            if split_vec.len() < 2 {
                panic!(
                    "Element {} must conform to format [type] [grade: 1-4]",
                    element_string
                );
            }
            let element = split_vec[0];
            let grade = split_vec[1];
            if element == self.element_type {
                match grade {
                    "1" => element_qty += 5,
                    "2" => element_qty += 10,
                    "3" => element_qty += 15,
                    "4" => element_qty += 25,
                    _ => panic!("Unknown element grade {}", grade),
                }
            }
        }
        return element_qty;
    }

    pub fn calculate_spirit_qty(&self, spirit_name: String) -> u8 {
        let spirit_qty = u8::try_from(
            self.spirits_socketed
                .iter()
                .filter(|x| **x == spirit_name)
                .count(),
        )
        .unwrap_or_default();

        return spirit_qty;
    }

    // pub fn calculate_attack_modifier(
    //     &mut self,
    //     hero_skill_map: &HashMap<String, HeroSkill>,
    //     class_innate_skill_names_map: &HashMap<String, String>,
    //     innate_skill_map: &HashMap<String, InnateSkill>,
    // ) {
    //     let mut attack_modifier = 0.0f64;

    //     let innate_skill_name = self.calculate_innate_skill_name(class_innate_skill_names_map);
    //     let innate_skill = innate_skill_map[&innate_skill_name].clone();

    //     attack_modifier += innate_skill.get_attack_percent();

    //     for skill_name in &self.skills {
    //         if !hero_skill_map.contains_key(skill_name) {
    //             panic!(
    //                 "Skill {} could not be found in keys for hero_skill_map",
    //                 skill_name
    //             );
    //         }
    //         let skill = hero_skill_map[skill_name].clone();
    //         attack_modifier += skill.get_attack_percent();
    //     }

    //     self.atk_modifier = attack_modifier;
    // }

    // pub fn calculate_defense_modifier(
    //     &mut self,
    //     hero_skill_map: &HashMap<String, HeroSkill>,
    //     class_innate_skill_names_map: &HashMap<String, String>,
    //     innate_skill_map: &HashMap<String, InnateSkill>,
    // ) {
    //     let mut defense_modifier = 0.0f64;

    //     let innate_skill_name = self.calculate_innate_skill_name(class_innate_skill_names_map);
    //     let innate_skill = innate_skill_map[&innate_skill_name].clone();

    //     defense_modifier += innate_skill.get_defense_percent();

    //     for skill_name in &self.skills {
    //         if !hero_skill_map.contains_key(skill_name) {
    //             panic!(
    //                 "Skill {} could not be found in keys for hero_skill_map",
    //                 skill_name
    //             );
    //         }
    //         let skill = hero_skill_map[skill_name].clone();
    //         defense_modifier += skill.get_defense_percent();
    //     }

    //     self.def_modifier = defense_modifier;
    // }

    pub fn scale_by_class(&mut self, hero_classes: &HashMap<String, HeroClass>) {
        if !hero_classes.contains_key(&self.class) {
            panic!(
                "Encountered unknown class {} for hero {}",
                self.class, self.identifier
            );
        }
        let class = hero_classes.get(&self.class).unwrap();

        let level_index = usize::from(self.level - 1);
        self.hp = class.base_hp[level_index];
        self.atk = class.base_atk[level_index];
        self.def = class.base_def[level_index];
        self.eva = class.base_eva;
        self.crit_chance = class.base_crit_chance;
        self.crit_mult = class.base_crit_mult;
        self.element_type = class.element_type.to_string();
    }

    pub fn calculate_stat_improvements_from_gear_and_skills(
        &mut self,
        bp_map: &HashMap<String, Blueprint>,
        hero_skill_map: &HashMap<String, HeroSkill>,
        class_innate_skill_names_map: &HashMap<String, String>,
        innate_skill_map: &HashMap<String, InnateSkill>,
    ) {
        let mut blueprints: Vec<Blueprint> = Default::default();
        for equip_name in &self.equipment_equipped {
            blueprints.push(bp_map[equip_name].clone());
        }

        let mut bonus_atk_percent = 0.0f64;
        let mut bonus_atk_value = 0.0f64;
        let mut bonus_hp_percent = 0.0f64;
        let mut bonus_hp_value = 0.0f64;
        let mut bonus_def_percent = 0.0f64;
        let mut bonus_def_value = 0.0f64;
        let mut bonus_eva_percent = 0.0f64;
        let mut bonus_crit_chance_percent = 0.0f64;
        let mut bonus_crit_damage_percent = 0.0f64;
        let mut bonus_rest_time_percent = 0.0f64;
        let mut bonus_xp_percent = 0.0f64;
        let mut bonus_survive_fatal_blow_chance_percent = 0.0f64;

        let mut spirit_bonus_atk_value: f64 = 0.0;
        let mut spirit_bonus_atk_percent: f64 = 0.0;
        let mut spirit_bonus_def_value: f64 = 0.0;
        let mut spirit_bonus_def_percent: f64 = 0.0;
        let mut spirit_bonus_hp_value: f64 = 0.0;
        let mut spirit_bonus_hp_percent: f64 = 0.0;
        let mut spirit_bonus_eva_percent: f64 = 0.0;
        let mut spirit_bonus_crit_dmg_percent: f64 = 0.0;
        let mut spirit_bonus_crit_chance_percent: f64 = 0.0;

        // Calculate gear bonuses
        for (gear_index, blueprint) in blueprints.iter().enumerate() {
            let mut bonus_item_all_stats_percent = 0.0f64;
            let mut bonus_item_atk_percent = 0.0f64;
            let mut bonus_item_def_percent = 0.0f64;

            // Check for skills that give bonus stats to gear
            for skill_name in &self.skills {
                if !hero_skill_map.contains_key(skill_name) {
                    panic!(
                        "Skill {} could not be found in keys for hero_skill_map",
                        skill_name
                    );
                }
                let skill = hero_skill_map[skill_name].clone();

                // Get all stats bonus if applicable
                bonus_item_all_stats_percent += skill.get_bonus_stats_from_all_equipment_percent();

                if skill.get_item_types().len() > 0 {
                    // Has bonuses associated with atleast one item type
                    for itype in skill.get_item_types() {
                        if blueprint.get_type() == itype {
                            // Have that type equipped, apply bonus(es)
                            bonus_item_atk_percent += skill.get_attack_with_item_percent();
                            bonus_item_def_percent += skill.get_defense_with_item_percent();
                        }
                    }
                }
            }

            let gear_quality = self.equipment_quality[gear_index].as_str();
            let gear_quality_bonus: f64;
            match gear_quality {
                "Normal" => gear_quality_bonus = 1.0,
                "Superior" => gear_quality_bonus = 1.25,
                "Flawless" => gear_quality_bonus = 1.5,
                "Epic" => gear_quality_bonus = 2.0,
                "Legendary" => gear_quality_bonus = 3.0,
                _ => panic!("Unknown gear_quality {}", gear_quality),
            }

            let gear_element = &self.elements_socketed[gear_index];
            let gear_element_split = gear_element.split_whitespace().collect::<Vec<&str>>();
            let gear_element_tier = gear_element_split[1].parse::<u8>().unwrap();

            let mut gear_element_atk_bonus: f64;
            let mut gear_element_def_bonus: f64;
            let mut gear_element_hp_bonus: f64;

            match gear_element_tier {
                1u8 => {
                    // Check 5 / Tier 5 (Luxurious)
                    if *gear_element == String::from("Luxurious 1") {
                        gear_element_atk_bonus = 26.0;
                        gear_element_def_bonus = 18.0;
                        gear_element_hp_bonus = 5.0;
                    } else {
                        gear_element_atk_bonus = 14.0;
                        gear_element_def_bonus = 10.0;
                        gear_element_hp_bonus = 3.0;
                    }
                }
                2u8 => {
                    gear_element_atk_bonus = 38.0;
                    gear_element_def_bonus = 25.0;
                    gear_element_hp_bonus = 8.0;
                }
                3u8 => {
                    // Check 15 / Tier 10 (Opulent)
                    if *gear_element == String::from("Opulent 3") {
                        gear_element_atk_bonus = 63.0;
                        gear_element_def_bonus = 42.0;
                        gear_element_hp_bonus = 13.0;
                    } else {
                        gear_element_atk_bonus = 48.0;
                        gear_element_def_bonus = 32.0;
                        gear_element_hp_bonus = 10.0;
                    }
                }
                4u8 => {
                    gear_element_atk_bonus = 89.0;
                    gear_element_def_bonus = 59.0;
                    gear_element_hp_bonus = 18.0;
                }
                _ => panic!("Unknown gear_element_tier {}", gear_element_tier),
            }
            let element_affinity = blueprint.get_elemental_affinity();
            if element_affinity.as_str() == gear_element_split[0] {
                gear_element_atk_bonus *= 1.5;
                gear_element_def_bonus *= 1.5;
                gear_element_hp_bonus *= 1.5;
            }

            let gear_spirit = &self.spirits_socketed[gear_index];
            let gear_spirit_split = gear_spirit.split_whitespace().collect::<Vec<&str>>();
            let gear_spirit_name = gear_spirit_split[0];
            let gear_spirit_tier = gear_spirit_split[1];

            let spirit_affinity = blueprint.get_spirit_affinity();

            let mut gear_spirit_atk_bonus: f64;
            let mut gear_spirit_def_bonus: f64;
            let mut gear_spirit_hp_bonus: f64;

            match gear_spirit_tier {
                "T4" => {
                    // Low-Tier Spirits
                    gear_spirit_atk_bonus = 16.0;
                    gear_spirit_def_bonus = 11.0;
                    gear_spirit_hp_bonus = 3.0;
                }
                "T5" => {
                    // Xolotl Spirit
                    gear_spirit_atk_bonus = 26.0;
                    gear_spirit_def_bonus = 18.0;
                    gear_spirit_hp_bonus = 5.0;
                }
                "T7" => {
                    // Mid-Tier Spirits
                    gear_spirit_atk_bonus = 41.0;
                    gear_spirit_def_bonus = 27.0;
                    gear_spirit_hp_bonus = 8.0;
                }
                "T9" => {
                    // High-Tier Spirits
                    gear_spirit_atk_bonus = 48.0;
                    gear_spirit_def_bonus = 32.0;
                    gear_spirit_hp_bonus = 10.0;
                }
                "TM" => {
                    // Mundra Spirit
                    gear_spirit_atk_bonus = 50.0;
                    gear_spirit_def_bonus = 33.0;
                    gear_spirit_hp_bonus = 10.0;
                }
                "T11" => {
                    // Quetzalcoatl Spirit
                    gear_spirit_atk_bonus = 63.0;
                    gear_spirit_def_bonus = 42.0;
                    gear_spirit_hp_bonus = 13.0; // only gives 10 on banana gun T6? only 6 on T5 imperial scutum? 10 on T5 silver thistle?? must be the min stuff from ress' sheet
                }
                "T12" => {
                    // Max-Tier Spirits
                    gear_spirit_atk_bonus = 89.0;
                    gear_spirit_def_bonus = 59.0;
                    gear_spirit_hp_bonus = 18.0;
                }
                _ => panic!("Unknown gear_spirit_tier {}", gear_spirit_tier),
            }
            match gear_spirit_name {
                "Wolf" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_atk_percent = 0.1;
                    } else {
                        spirit_bonus_atk_percent = 0.05;
                    }
                }
                "Ram" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_def_percent = 0.1;
                    } else {
                        spirit_bonus_def_percent = 0.05;
                    }
                }
                "Eagle" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_crit_chance_percent = 0.03;
                    } else {
                        spirit_bonus_crit_chance_percent = 0.02;
                    }
                }
                "Ox" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_hp_percent = 0.05;
                    } else {
                        spirit_bonus_hp_percent = 0.03;
                    }
                }
                "Viper" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_crit_dmg_percent = 0.2;
                    } else {
                        spirit_bonus_crit_dmg_percent = 0.15;
                    }
                }
                "Cat" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_eva_percent = 0.03;
                    } else {
                        spirit_bonus_eva_percent = 0.02;
                    }
                }
                "Bear" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_atk_percent = 0.07;
                        spirit_bonus_hp_value = 20.0;
                    } else {
                        spirit_bonus_atk_percent = 0.05;
                        spirit_bonus_hp_value = 15.0;
                    }
                }
                "Walrus" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_hp_percent = 0.08;
                    } else {
                        spirit_bonus_hp_percent = 0.05;
                    }
                }
                "Mammoth" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_def_percent = 0.13;
                    } else {
                        spirit_bonus_def_percent = 0.1;
                    }
                }
                "Lion" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_atk_percent = 0.07;
                        spirit_bonus_eva_percent = 0.02;
                    } else {
                        spirit_bonus_atk_percent = 0.05;
                        spirit_bonus_eva_percent = 0.01;
                    }
                }
                "Tiger" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_def_percent = 0.07;
                        spirit_bonus_eva_percent = 0.02;
                    } else {
                        spirit_bonus_def_percent = 0.05;
                        spirit_bonus_eva_percent = 0.01;
                    }
                }
                "Phoenix" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_hp_percent = 0.05;
                    } else {
                        spirit_bonus_hp_percent = 0.04;
                    }
                }
                "Hydra" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_def_value = 125.0;
                        spirit_bonus_hp_value = 35.0;
                    } else {
                        spirit_bonus_def_value = 100.0;
                        spirit_bonus_hp_value = 25.0;
                    }
                }
                "Tarrasque" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_def_percent = 0.25;
                    } else {
                        spirit_bonus_def_percent = 0.2;
                    }
                }
                "Carbuncle" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_crit_chance_percent = 0.03;
                        spirit_bonus_eva_percent = 0.03;
                    } else {
                        spirit_bonus_crit_chance_percent = 0.02;
                        spirit_bonus_eva_percent = 0.02;
                    }
                }
                "Chimera" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_atk_percent = 0.15;
                        spirit_bonus_crit_dmg_percent = 0.15;
                    } else {
                        spirit_bonus_atk_percent = 0.1;
                        spirit_bonus_crit_dmg_percent = 0.1;
                    }
                }
                "Kraken" => {
                    if spirit_affinity.as_str() == gear_spirit_name {
                        spirit_bonus_atk_value = 125.0;
                        spirit_bonus_atk_percent = 0.15;
                    } else {
                        spirit_bonus_atk_value = 100.0;
                        spirit_bonus_atk_percent = 0.1;
                    }
                }
                _ => (),
            }

            if spirit_affinity.as_str() == gear_spirit_name {
                gear_spirit_atk_bonus *= 1.5;
                gear_spirit_def_bonus *= 1.5;
                gear_spirit_hp_bonus *= 1.5;
            }

            // Calculate and apply gear bonus to running totals
            let item_attack_final = ((blueprint.get_atk() * gear_quality_bonus)
                + f64::min(gear_element_atk_bonus, blueprint.get_atk())
                + f64::min(gear_spirit_atk_bonus, blueprint.get_atk()))
                * (1.0 + bonus_item_atk_percent + bonus_item_all_stats_percent);
            let item_defense_final = ((blueprint.get_def() * gear_quality_bonus)
                + f64::min(gear_element_def_bonus, blueprint.get_def())
                + f64::min(gear_spirit_def_bonus, blueprint.get_def()))
                * (1.0 + bonus_item_def_percent + bonus_item_all_stats_percent);
            let item_hp_final = ((blueprint.get_hp() * gear_quality_bonus)
                + f64::min(gear_element_hp_bonus, blueprint.get_hp())
                + f64::min(gear_spirit_hp_bonus, blueprint.get_hp()))
                * (1.0 + bonus_item_all_stats_percent);
            // bonus_atk_value += blueprint.get_atk() * gear_quality_bonus * (1.0 + bonus_item_atk_percent + bonus_item_all_stats_percent);
            // bonus_def_value += blueprint.get_def() * gear_quality_bonus * (1.0 + bonus_item_def_percent + bonus_item_all_stats_percent);
            // bonus_hp_value += blueprint.get_hp() * gear_quality_bonus * (1.0 + bonus_item_all_stats_percent);
            bonus_atk_value += item_attack_final;
            bonus_def_value += item_defense_final;
            bonus_hp_value += item_hp_final;
            bonus_eva_percent += blueprint.get_eva() * (1.0 + bonus_item_all_stats_percent);
            bonus_crit_chance_percent +=
                blueprint.get_crit() * (1.0 + bonus_item_all_stats_percent);
        }

        // Calculate hero-wide skill bonuses
        let mut skill_bonus_atk_percent: f64 = 0.0;
        let mut skill_bonus_atk_value: f64 = 0.0;
        let mut skill_bonus_hp_percent: f64 = 0.0;
        let mut skill_bonus_hp_value: f64 = 0.0;
        let mut skill_bonus_def_percent: f64 = 0.0;
        let mut skill_bonus_eva_percent: f64 = 0.0;
        let mut skill_bonus_crit_chance_percent: f64 = 0.0;
        let mut skill_bonus_crit_damage_percent: f64 = 0.0;
        let mut skill_bonus_rest_time_percent: f64 = 0.0;
        let mut skill_bonus_xp_percent_percent: f64 = 0.0;
        let mut skill_bonus_survive_fatal_blow_chance_percent: f64 = 0.0;

        for skill_name in &self.skills {
            if !hero_skill_map.contains_key(skill_name) {
                panic!(
                    "Skill {} could not be found in keys for hero_skill_map",
                    skill_name
                );
            }
            let skill = hero_skill_map[skill_name].clone();

            skill_bonus_atk_percent += skill.get_attack_percent();
            skill_bonus_atk_value += skill.get_attack_value();
            skill_bonus_hp_percent += skill.get_hp_percent();
            skill_bonus_hp_value += skill.get_hp_value();
            skill_bonus_def_percent += skill.get_defense_percent();
            skill_bonus_eva_percent += skill.get_evasion_percent();
            skill_bonus_crit_chance_percent += skill.get_crit_chance_percent();
            skill_bonus_crit_damage_percent += skill.get_crit_damage_percent();
            skill_bonus_rest_time_percent += skill.get_rest_time_percent();
            skill_bonus_xp_percent_percent += skill.get_xp_percent();
            skill_bonus_survive_fatal_blow_chance_percent +=
                skill.get_survive_fatal_blow_chance_percent();
        }

        let mut geo_astramancer_element_qty_or_chieftain_threat_bonus: f64 = 0.0;
        match self.class.as_str() {
            "Geomancer" => {
                geo_astramancer_element_qty_or_chieftain_threat_bonus =
                    f64::from(self.calculate_element_qty())
            }
            "Astramancer" => {
                geo_astramancer_element_qty_or_chieftain_threat_bonus =
                    f64::from(self.calculate_element_qty())
            }
            "Chieftain" => {
                geo_astramancer_element_qty_or_chieftain_threat_bonus =
                    0.4 * f64::from(self.threat_rating)
            }
            _ => (),
        }

        // ATK calc
        let base_atk = self.atk;
        let seeded_atk = base_atk + f64::from(self.hp_seeds * 4);
        let summarized_base_atk_value = seeded_atk + spirit_bonus_atk_value + skill_bonus_atk_value;
        let summarized_atk_percent_modifier = 1.0
            + skill_bonus_atk_percent
            + geo_astramancer_element_qty_or_chieftain_threat_bonus
            + spirit_bonus_atk_percent;
        let modified_atk_value = summarized_base_atk_value * summarized_atk_percent_modifier;
        let modified_atk_bonus = bonus_atk_value * summarized_atk_percent_modifier;
        let new_atk = modified_atk_value + modified_atk_bonus;
        self.atk = new_atk;
        // ((seeded_atk + gear_spirit_bonus_atk_value + sum(skill_bonus_atk_value)) * (1 + ((skill_atk_percent + geo_astramancer_element_qty_or_chieftain_threat_bonus) + bonus_spirit_atk_percent)/100)) + (bonus_atk_value * (1 + ((skill_atk_percent + geo_astramancer_element_qty_or_chieftain_threat_bonus) + bonus_spirit_atk_percent)/100)))

        // ATK mod calc
        let new_atk_mod = skill_bonus_atk_percent
            + geo_astramancer_element_qty_or_chieftain_threat_bonus
            + spirit_bonus_atk_percent;
        self.atk_modifier = new_atk_mod;
        // (skill_atk_percent + geo_astramancer_element_qty_or_chieftain_threat_bonus) + bonus_spirit_atk_percent)
    }

    pub fn _round_floats_for_display(&self) -> Hero {
        let mut h2 = self.clone();
        h2.hp = round_to_2(h2.hp);
        h2.atk = round_to_2(h2.atk);
        h2.def = round_to_2(h2.def);
        h2.eva = round_to_2(h2.eva);
        h2.crit_chance = round_to_2(h2.crit_chance);
        h2.crit_mult = round_to_2(h2.crit_mult);
        return h2;
    }
}

impl From<Hero> for SimHero {
    /// Create a hero from the input object performing type validation and calculating certain fields
    fn from(item: Hero) -> Self {
        let i2 = item.clone();
        return create_sim_hero(
            item.identifier,
            item.class,
            item.level,
            item.rank,
            item.innate_tier,
            item.hp,
            item.atk,
            item.def,
            item.threat_rating,
            item.crit_chance,
            item.crit_mult,
            item.eva,
            i2.calculate_element_qty(),
            item.element_type,
            i2.calculate_spirit_qty(String::from("Armadillo T7")),
            i2.calculate_spirit_qty(String::from("Lizard T7")),
            i2.calculate_spirit_qty(String::from("Shark T9")),
            i2.calculate_spirit_qty(String::from("Dinosaur T9")),
            i2.calculate_spirit_qty(String::from("Mundra T10")),
            item.atk_modifier,
            item.def_modifier,
        )
        .unwrap();
    }
}

impl From<Hero> for HeroInput {
    fn from(item: Hero) -> Self {
        return create_hero_input(
            item.identifier,
            item.class,
            item.level,
            item.rank,
            item.hp,
            item.atk,
            item.def,
            item.eva,
            item.crit_chance,
            item.crit_mult,
            item.threat_rating,
            item.element_type,
            item.hp_seeds,
            item.atk_seeds,
            item.def_seeds,
            item.skills,
            item.equipment_equipped,
            item.equipment_quality,
            item.elements_socketed,
            item.spirits_socketed,
        );
    }
}
