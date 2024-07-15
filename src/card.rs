use std::path::{Path, PathBuf};

use bevy::{prelude::*, utils::HashMap};
use bevy_svg::prelude::*;
use cache_bust::asset;
use strum::{EnumIter, IntoEnumIterator};

mod asset_names;
use asset_names::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
pub enum Suit {
	Spades,
	Clubs,
	Diamonds,
	Hearts,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, EnumIter, Debug)]
pub enum Value {
	Ace,
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	Jack,
	Queen,
	King,
}

impl Value {
	pub fn as_number(&self) -> u8 {
		use Value::*;
		match self {
			Ace => 1,
			Two => 2,
			Three => 3,
			Four => 4,
			Five => 5,
			Six => 6,
			Seven => 7,
			Eight => 8,
			Nine => 9,
			Ten => 10,
			Jack => 11,
			Queen => 12,
			King => 13,
		}
	}
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Card {
	pub suit: Suit,
	pub value: Value,
}

impl Card {
	pub fn deck() -> impl Iterator<Item=Card> {
		Suit::iter()
			.flat_map(|suit| Value::iter()
				.map(move |value| Card {
					suit,
					value,
				})
			)
	}
}

fn load_card_assets(path: &Path, asset_server: &AssetServer) -> HashMap<Card, Handle<Svg>> {
	Suit::iter()
		.flat_map(|suit| {
			Value::iter()
				.map(move |value| Card {
					suit,
					value,
				})
		})
		.map(|card| (card, asset_server.load(path.join(asset_name(card)))))
		.collect()
}

#[derive(Resource, Debug)]
pub struct CardAssets {
	cards: HashMap<Card, Handle<Svg>>,
	slot: Handle<Svg>,
	black_back: Handle<Svg>,
	red_back: Handle<Svg>,
}

impl CardAssets {
	pub fn get(&self, card: Card) -> Handle<Svg> {
		self.cards.get(&card).cloned().expect("All possible cards have been inserted")
	}
	
	pub fn get_back(&self, suit: Suit) -> Handle<Svg> {
		use Suit::*;
		match suit {
			Spades | Clubs => self.black_back.clone(),
			Diamonds | Hearts => self.red_back.clone(),
		}
	}
	
	pub fn slot(&self) -> Handle<Svg> {
		self.slot.clone()
	}
}

pub struct CardAssetsPlugin(pub PathBuf);

impl Plugin for CardAssetsPlugin {
	fn build(&self, app: &mut App) {
		let asset_server: &AssetServer = app.world.get_resource().expect("AssetServer must be initialized");
		let slot = asset_server.load(self.0.join(asset!("slot.svg")));
		let black_back = asset_server.load(self.0.join(asset!("1B.svg")));
		let red_back = asset_server.load(self.0.join(asset!("2B.svg")));
		
		let card_assets = CardAssets {
			cards: load_card_assets(&self.0, asset_server),
			slot,
			black_back,
			red_back,
		};
		
		app.insert_resource(card_assets);
	}
}
