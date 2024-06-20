use std::path::{Path, PathBuf};

use bevy::{prelude::*, utils::HashMap};
use bevy_svg::prelude::*;
use strum::{EnumIter, IntoEnumIterator};

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

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Card {
	pub suit: Suit,
	pub value: Value,
}

fn asset_path(path: &Path, card: Card) -> PathBuf {
	use Suit::*;
	use Value::*;
	
	let suit_char = match card.suit {
		Spades => 'S',
		Clubs => 'C',
		Diamonds => 'D',
		Hearts => 'H',
	};
	
	let value_char = match card.value {
		Ace => 'A',
		Two => '2',
		Three => '3',
		Four => '4',
		Five => '5',
		Six => '6',
		Seven => '7',
		Eight => '8',
		Nine => '9',
		Ten => 'T',
		Jack => 'J',
		Queen => 'Q',
		King => 'K',
	};
	
	path.join(format!("{value_char}{suit_char}.svg"))
}

fn load_assets(path: &Path, asset_server: &AssetServer) -> HashMap<Card, Handle<Svg>> {
	Suit::iter()
		.flat_map(|suit| {
			Value::iter()
				.map(move |value| Card {
					suit,
					value,
				})
		})
		.map(|card| (card, asset_server.load(asset_path(path, card))))
		.collect()
}

#[derive(Resource, Debug)]
pub struct CardAssets(HashMap<Card, Handle<Svg>>);

impl CardAssets {
	pub fn get(&self, card: Card) -> Handle<Svg> {
		self.0.get(&card).cloned().expect("All possible cards have been inserted")
	}
}

pub struct CardAssetsPlugin(pub PathBuf);

impl Plugin for CardAssetsPlugin {
	fn build(&self, app: &mut App) {
		let asset_server: &AssetServer = app.world.get_resource().expect("AssetServer must be initialized");
		let card_assets = CardAssets(load_assets(&self.0, asset_server));
		app.insert_resource(card_assets);
	}
}
