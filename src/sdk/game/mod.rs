use std::{path::Path, sync::Arc};

use anyhow::Result;
use derive_new::new;
use jni::{JNIEnv, signature::{JavaType, ReturnType}, sys::{jobject, jdouble}, objects::{JObject, JValueGen}};
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

macro_rules! new {
    ($env:expr, $class:expr, $sig:expr, $args:expr) => {
        $env.new_object(
            class!($env, $class),
            MAPPINGS.remap_descriptor($sig),
            $args
        )?
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

    pub fn get_player(&mut self) -> Result<ClientPlayerEntity<'a>> {
        let player_obj = get_field!(self.env, &self.mc, "net/minecraft/client/MinecraftClient", "player", "Lnet/minecraft/client/network/ClientPlayerEntity;").l()?;

        let player = ClientPlayerEntity::new(player_obj, unsafe { self.env.unsafe_clone() });
        Ok(player)
    }

}

#[derive(new)]
pub struct ClientPlayerEntity<'a> {
    jobj: JObject<'a>,
    env: JNIEnv<'a>
}

impl<'a> ClientPlayerEntity<'a> {
    // Player is a type of Entity
    pub fn as_entity(self) -> Result<Entity<'a>> {
        let entity = Entity::new(self.jobj, unsafe { self.env.unsafe_clone() });
        Ok(entity)
    }
}

#[derive(new)]
pub struct Entity<'a> {
    entity: JObject<'a>,
    env: JNIEnv<'a>
}

impl<'a> Entity<'a> {

    // Lnet/minecraft/entity/Entity;getPos()Lnet/minecraft/util/math/Vec3d;
    pub fn get_pos(&mut self) -> Result<Vec3d<'a>> {
        let pos = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "getPos", "()Lnet/minecraft/util/math/Vec3d;", &[]).l()?;
        let pos = Vec3d::new(pos, unsafe { self.env.unsafe_clone() });
        Ok(pos)
    }

    // Lnet/minecraft/entity/Entity;getX()D
    pub fn get_x(&mut self) -> Result<jdouble> {
        let x = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "getX", "()D", &[]).d()?;
        Ok(x)
    }

    // Lnet/minecraft/entity/Entity;getY()D
    pub fn get_y(&mut self) -> Result<jdouble> {
        let y = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "getY", "()D", &[]).d()?;
        Ok(y)
    }

    // Lnet/minecraft/entity/Entity;getZ()D
    pub fn get_z(&mut self) -> Result<jdouble> {
        let z = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "getZ", "()D", &[]).d()?;
        Ok(z)
    }

    // Lnet/minecraft/entity/Entity;getVelocity()Lnet/minecraft/util/math/Vec3d;
    pub fn get_velocity(&mut self) -> Result<Vec3d<'a>> {
        let velocity = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "getVelocity", "()Lnet/minecraft/util/math/Vec3d;", &[]).l()?;
        let velocity = Vec3d::new(velocity, unsafe { self.env.unsafe_clone() });
        Ok(velocity)
    }

    // Lnet/minecraft/entity/Entity;setVelocity(Lnet/minecraft/util/math/Vec3d;)V
    pub fn set_velocity(&mut self, velocity: &Vec3d) -> Result<()> {
        let args = &[JValueGen::Object(&velocity.jobj)];
        call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "setVelocity", "(Lnet/minecraft/util/math/Vec3d;)V", args);
        Ok(())
    }

    // Lnet/minecraft/entity/Entity;addVelocity(Lnet/minecraft/util/math/Vec3d;)V
    pub fn add_velocity(&mut self, velocity: &Vec3d) -> Result<()> {
        let args = &[JValueGen::Object(&velocity.jobj)];
        call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "addVelocity", "(Lnet/minecraft/util/math/Vec3d;)V", args);
        Ok(())
    }

    // Lnet/minecraft/entity/Entity;isOnGround()Z
    pub fn is_on_ground(&mut self) -> Result<bool> {
        let is_on_ground = call_method!(self.env, &self.entity, "net/minecraft/entity/Entity", "isOnGround", "()Z", &[]).z()?;
        Ok(is_on_ground)
    }

}

/// Vec3d JNI wrapper
/// 
/// net/minecraft/util/math/Vec3d
#[derive(new)]
pub struct Vec3d<'a> {
    jobj: JObject<'a>,
    env: JNIEnv<'a>
}

impl <'a> Vec3d<'a> {

    // Lnet/minecraft/util/math/Vec3d;<init>(DDD)V
    pub fn new_obj(env: JNIEnv<'a>, x: f64, y: f64, z: f64) -> Result<Vec3d<'a>> {
        let obj: JObject<'a> = new!(unsafe { env.unsafe_clone() }, "net/minecraft/util/math/Vec3d", "(DDD)V",
            &[x.into(), y.into(), z.into()]);
        Ok(Vec3d::new(obj, env))
    }

    // Lnet/minecraft/util/math/Vec3d;normalize()Lnet/minecraft/util/math/Vec3d;
    pub fn normalize(&mut self) -> Result<Vec3d<'a>> {
        let normalized = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Vec3d", "normalize", "()Lnet/minecraft/util/math/Vec3d;", &[]).l()?;
        let normalized = Vec3d::new(normalized, unsafe { self.env.unsafe_clone() });
        Ok(normalized)
    }

    // Lnet/minecraft/util/math/Vec3d;add(Lnet/minecraft/util/math/Vec3d;)Lnet/minecraft/util/math/Vec3d;
    pub fn add(&mut self, other: &Vec3d) -> Result<Vec3d<'a>> {
        let args = &[JValueGen::Object(&other.jobj)];
        let added = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Vec3d", "add", "(Lnet/minecraft/util/math/Vec3d;)Lnet/minecraft/util/math/Vec3d;", args).l()?;
        let added = Vec3d::new(added, unsafe { self.env.unsafe_clone() });
        Ok(added)
    }

    // Lnet/minecraft/util/math/Vec3d;add(DDD)Lnet/minecraft/util/math/Vec3d;
    pub fn add_xyz(&mut self, x: f64, y: f64, z: f64) -> Result<Vec3d<'a>> {
        let args = &[x.into(), y.into(), z.into()];
        let added = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Vec3d", "add", "(DDD)Lnet/minecraft/util/math/Vec3d;", args).l()?;
        let added = Vec3d::new(added, unsafe { self.env.unsafe_clone() });
        Ok(added)
    }

    // todo: replace with a cast function
    pub fn as_position(self) -> Result<Position<'a>> {
        let position = Position::new(self.jobj, unsafe { self.env.unsafe_clone() });
        Ok(position)
    }

}

// net/minecraft/util/math/Position
#[derive(new)]
pub struct Position<'a> {
    jobj: JObject<'a>,
    env: JNIEnv<'a>
}

impl <'a> Position<'a> {

    // Lnet/minecraft/util/math/Position;getX()D
    pub fn get_x(&mut self) -> Result<jdouble> {
        let x = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Position", "getX", "()D", &[]).d()?;
        Ok(x)
    }

    // Lnet/minecraft/util/math/Position;getY()D
    pub fn get_y(&mut self) -> Result<jdouble> {
        let y = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Position", "getY", "()D", &[]).d()?;
        Ok(y)
    }

    // Lnet/minecraft/util/math/Position;getZ()D
    pub fn get_z(&mut self) -> Result<jdouble> {
        let z = call_method!(self.env, &self.jobj, "net/minecraft/util/math/Position", "getZ", "()D", &[]).d()?;
        Ok(z)
    }

}