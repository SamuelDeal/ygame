use rand::{prelude::SliceRandom, Rng};

pub fn generate_name() -> String {
	let animal = ANIMALS_DICT.choose(&mut rand::thread_rng()).expect("No animal found");
	let adj = ADJECTIVE_DICT
		.choose(&mut rand::thread_rng())
		.expect("No adjective found");
	match animal.gender {
		AnimalGender::Feminine if adj.fem.is_some() => animal.name.to_owned() + " " + adj.fem.unwrap(),
		_ => animal.name.to_owned() + " " + adj.masc,
	}
}

#[derive(PartialEq)]
enum AnimalGender {
	Masculine,
	Feminine,
}

struct AnimalName {
	pub name: &'static str,
	pub gender: AnimalGender,
}

struct Adjective {
	pub masc: &'static str,
	pub fem: Option<&'static str>,
}

const ANIMALS_DICT: [AnimalName; 53] = [
	AnimalName {
		name: "L'aigle",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "L'âne",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "L'ânesse",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La baleine",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La belette",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le bouc",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La chèvre",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le bœuf",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le taureau",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La vache",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le canard",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La cane",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La carpe",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le castor",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le cerf",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La biche",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le chat",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La chatte",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le cheval",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La jument",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La chouette",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le coq",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La poule",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le coyote",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le dindon",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La dinde",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "L'écureuil ",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La gazelle",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le gorille",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La grenouille",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La guêpe",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le hérisson",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le hibou",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "L'hirondelle",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "L'oie",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le loup",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La louve",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le lynx",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La marmotte",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le bélier",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le mouton",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La brebis",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le panda",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le renard",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le rossignol",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "Le serpent",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La sourie",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le rat",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La rate",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le tigre",
		gender: AnimalGender::Masculine,
	},
	AnimalName {
		name: "La tigresse",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "La tourterelle",
		gender: AnimalGender::Feminine,
	},
	AnimalName {
		name: "Le zèbre",
		gender: AnimalGender::Masculine,
	},
];

const ADJECTIVE_DICT: [Adjective; 22] = [
	Adjective {
		masc: "confiant",
		fem: Some("confiante"),
	},
	Adjective {
		masc: "courageux",
		fem: Some("courageuse"),
	},
	Adjective {
		masc: "égoïste",
		fem: None,
	},
	Adjective {
		masc: "gigantesque",
		fem: None,
	},
	Adjective {
		masc: "idéaliste",
		fem: None,
	},
	Adjective {
		masc: "intelligent",
		fem: Some("intelligente"),
	},
	Adjective {
		masc: "inventif",
		fem: Some("inventive"),
	},
	Adjective {
		masc: "optimiste",
		fem: None,
	},
	Adjective {
		masc: "prétentieux",
		fem: Some("prétentieuse"),
	},
	Adjective {
		masc: "majestueux",
		fem: Some("majestueuse"),
	},
	Adjective {
		masc: "passionnant",
		fem: Some("passionnante"),
	},
	Adjective {
		masc: "placide",
		fem: None,
	},
	Adjective {
		masc: "ébouriffé",
		fem: Some("ébouriffée"),
	},
	Adjective {
		masc: "enragé",
		fem: Some("enragée"),
	},
	Adjective {
		masc: "révolutionnaire",
		fem: None,
	},
	Adjective {
		masc: "somptueux",
		fem: Some("somptueuse"),
	},
	Adjective {
		masc: "festif",
		fem: Some("festive"),
	},
	Adjective {
		masc: "incrédule",
		fem: None,
	},
	Adjective {
		masc: "insupportable",
		fem: None,
	},
	Adjective {
		masc: "trépidant",
		fem: Some("trépidant"),
	},
	Adjective {
		masc: "vénérable",
		fem: None,
	},
	Adjective {
		masc: "valeureux",
		fem: Some("valeureuse"),
	},
];
