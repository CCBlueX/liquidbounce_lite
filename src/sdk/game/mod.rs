use std::{path::Path, sync::Arc};

use anyhow::Result;
use derive_new::new;
use jni::{JNIEnv, signature::{JavaType, ReturnType}, sys::{jobject, jdouble}, objects::JObject};
use lazy_static::lazy_static;
use yarn_remapper::{Mapping, parse_tiny_v2};

lazy_static!(
    // todo: replace with a better way to get the mappings
    static ref MAPPINGS: Mapping = parse_tiny_v2(Path::new("mappings.tiny")).unwrap();
);

macro_rules! class {
    ($env:expr, $class:expr) => {
        $env.find_class(MAPPINGS.remap_class($class).unwrap_or($class.to_string()))?
    };
}

macro_rules! call_method {
    ($env:expr, $obj:expr, $class:expr, $method:expr, $sig:expr, $args:expr) => {
        $env.call_method(
            $obj,
            MAPPINGS.remap_method($class, $method, $sig).unwrap_or($method.to_string()),
            MAPPINGS.remap_descriptor($sig),
            $args
        )?
    };
}

macro_rules! call_static_method {
    ($env:expr, $class:expr, $method:expr, $sig:expr, $args:expr) => {
        $env.call_static_method(
            class!($env, $class), 
            MAPPINGS.remap_method($class, $method, $sig).unwrap_or($method.to_string()), 
            MAPPINGS.remap_descriptor($sig), 
            $args
        )?
    };
}

macro_rules! get_field {
    ($env:expr, $obj:expr, $class:expr, $field:expr, $sig:expr) => {
        $env.get_field(
            $obj,
            MAPPINGS.remap_field($class, $field, $sig).unwrap_or($field.to_string()),
            MAPPINGS.remap_descriptor($sig)
        )?
    };
}

macro_rules! get_static_field {
    ($env:expr, $class:expr, $field:expr, $sig:expr) => {
        $env.get_static_field(
            class!($env, $class),
            MAPPINGS.remap_field($class, $field, $sig).unwrap_or($field.to_string()),
            MAPPINGS.remap_descriptor($sig)
        )?
    };
}

#[derive(new)]
pub struct MinecraftClient<'a> {
    env: JNIEnv<'a>,
    mc: JObject<'a>
}

impl <'a> MinecraftClient<'a> {

    pub fn get_instance(env: JNIEnv) -> Result<MinecraftClient> {
        let mc = call_static_method!(unsafe { env.unsafe_clone() }, "net/minecraft/client/MinecraftClient",
            "getInstance", "()Lnet/minecraft/client/MinecraftClient;", &[]);
        let mc_obj = mc.l()?;

        let minecraft_client = MinecraftClient::new(env, mc_obj);
        Ok(minecraft_client)
    }

    pub fn get_player(&mut self) -> Result<Player<'a>> {
        let player_obj = get_field!(self.env, &self.mc, "net/minecraft/client/MinecraftClient", "player", "Lnet/minecraft/client/network/ClientPlayerEntity;").l()?;

        let player = Player::new(player_obj, unsafe { self.env.unsafe_clone() });
        Ok(player)
    }

}

#[derive(new)]
pub struct Player<'a> {
    player: JObject<'a>,
    env: JNIEnv<'a>
}

impl<'a> Player<'a> {

    // Lnet/minecraft/entity/Entity;getX()D
    pub fn get_x(&mut self) -> Result<jdouble> {
        let x = call_method!(self.env, &self.player, "net/minecraft/entity/Entity", "getX", "()D", &[]).d()?;
        Ok(x)
    }

    // Lnet/minecraft/entity/Entity;getY()D
    pub fn get_y(&mut self) -> Result<jdouble> {
        let y = call_method!(self.env, &self.player, "net/minecraft/entity/Entity", "getY", "()D", &[]).d()?;
        Ok(y)
    }

    // Lnet/minecraft/entity/Entity;getZ()D
    pub fn get_z(&mut self) -> Result<jdouble> {
        let z = call_method!(self.env, &self.player, "net/minecraft/entity/Entity", "getZ", "()D", &[]).d()?;
        Ok(z)
    }

}