use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    utils::BoxedFuture,
};

use crate::components::{Sprite, StyleMap};

#[derive(Default)]
pub struct SpriteLoader;

impl AssetLoader for SpriteLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let string = std::str::from_utf8(bytes);
            let sprite = Sprite::new(string?);
            load_context.set_default_asset(LoadedAsset::new(sprite));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(Default)]
pub struct StyleMapLoader;

impl AssetLoader for StyleMapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let stylemap = ron::de::from_bytes::<StyleMap>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(stylemap));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["stylemap"]
    }
}
