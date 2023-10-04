use std::path::{Path, PathBuf};

type PokedexNumber = u16;
type Form = u8;
type ItemNumber = u16;

pub enum Shinyness {
    Shiny,
    Normal,
}

impl Shinyness {
    pub const fn get_shiny_marker(&self) -> &'static str {
        match self {
            Shinyness::Shiny => "s",
            Shinyness::Normal => "",
        }
    }
}

pub enum BattleSide {
    Front,
    Back,
}

impl BattleSide {
    pub const fn get_side_marker(&self) -> &'static str {
        match self {
            BattleSide::Back => "b",
            BattleSide::Front => "",
        }
    }
}

pub enum Asset {
    OverworldSprite(PokedexNumber, Form, Shinyness),
    BattleSprite(PokedexNumber, Form, Shinyness, BattleSide),
    BattleCry(PokedexNumber),
}


impl Asset {
    pub fn get_prefix(&self) -> PathBuf {
        match self {
            Asset::OverworldSprite(_, _, _) => Path::new("graphics").join("overworld_sprites"),
            Asset::BattleSprite(_, _, _, _) => Path::new("graphics").join("battle_sprites"),
            Asset::BattleCry(_) => Path::new("audio").join("cries"),
        }
    }

    pub fn get_path(&self) -> PathBuf {
        let prefix = self.get_prefix();

        let filename = match self {
            Asset::OverworldSprite(pokedex, form, shinyness) => {
                let shiny_str = shinyness.get_shiny_marker();
                let form_str = if *form > 0 {
                    format!("_{}", form)
                } else {
                    "".into()
                };

                format!("{:0>3}{}{}.png", pokedex, shiny_str, form_str)
            }
            Asset::BattleSprite(pokedex, form, shinyness, battle_side) => {
                let shiny_str = shinyness.get_shiny_marker();
                let side_marker = battle_side.get_side_marker();
                let form_str = if *form > 0 {
                    format!("_{}", form)
                } else {
                    "".into()
                };

                format!(
                    "{:0>3}{}{}{}.png",
                    pokedex, shiny_str, side_marker, form_str
                )
            }
            Asset::BattleCry(pokedex) => {
                format!("{:0>3}Cry.wav", pokedex)
            }
        };

        prefix.join(filename)
    }
}
