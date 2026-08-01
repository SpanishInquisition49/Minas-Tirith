use std::hash::{DefaultHasher, Hash, Hasher};

use iroh::PublicKey;

pub mod blobs;
pub mod identity;
pub mod library;
pub mod node;

const ADJECTIVES: &[&str] = &[
    "Ancient",
    "Ashen",
    "Bright",
    "Dusky",
    "Eastern",
    "Elder",
    "Fair",
    "Far",
    "Forgotten",
    "Golden",
    "Grey",
    "Hidden",
    "High",
    "Lonely",
    "Misty",
    "Moonlit",
    "Noble",
    "Northern",
    "Old",
    "Quiet",
    "Royal",
    "Sacred",
    "Shadow",
    "Silent",
    "Silver",
    "Starborn",
    "Starlit",
    "Steady",
    "Twilit",
    "Wandering",
    "White",
    "Woodland",
];

const NOUNS: &[&str] = &[
    "Bard",
    "Guardian",
    "Harbinger",
    "Herald",
    "Keeper",
    "King",
    "Knight",
    "Loremaster",
    "Pilgrim",
    "Ranger",
    "Sage",
    "Scholar",
    "Sentinel",
    "Smith",
    "Wanderer",
    "Warden",
    "Watcher",
    "Wayfarer",
    "Weaver",
    "Wizard",
    "Woodwright",
    "Seer",
    "Healer",
    "Steward",
    "Mariner",
    "Falconer",
    "Chronicler",
    "Guide",
    "Elder",
    "Thane",
    "Messenger",
    "Minstrel",
];

const EPITHETS: &[&str] = &[
    "of dawn",
    "of stars",
    "of ash",
    "of the west",
    "of the north",
    "of the east",
    "of the woods",
    "of the hills",
    "of the river",
    "of the mist",
    "of twilight",
    "of silver",
    "of gold",
    "of shadow",
    "of light",
    "of stone",
    "of wind",
    "of fire",
    "of oaks",
    "of cedars",
    "of the moon",
    "of the sky",
    "of the deep",
    "of the vale",
    "of the harbor",
    "of the forest",
    "of the peak",
    "of the path",
    "of the warden",
    "of the king",
    "the Wise",
    "the Grey",
];

pub trait PrettyDisplay {
    fn node_id(&self) -> PublicKey;

    /// Deterministically maps a NodeId's public key to a human-friendly name.
    /// Same key -> same name, every time, on every peer's screen.
    fn pretty_name(&self) -> String {
        let mut hasher = DefaultHasher::new();
        self.node_id().as_bytes().hash(&mut hasher);
        let hash = hasher.finish();

        let adjective = ADJECTIVES[(hash as usize) % ADJECTIVES.len()];
        let noun = NOUNS[((hash >> 10) as usize) % NOUNS.len()];
        let epithet = EPITHETS[((hash >> 20) as usize) % EPITHETS.len()];
        let suffix = hash & 0xFFF;

        format!("{adjective} {noun} {epithet} [{suffix:03x}]")
    }
}

pub struct NodeIdDisplay(pub PublicKey);

impl PrettyDisplay for NodeIdDisplay {
    fn node_id(&self) -> PublicKey {
        self.0
    }
}
