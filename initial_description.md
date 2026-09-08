# General Description

Plumefoil Racing is an efoil simulation multiplatform video game

# Main Requirements

Plumefoil Racing is a standalone game that should achieve several goals:

- be appealing to users and provide fun gameplay
- constitute a somewhat realistic foil simulation that helps players understand how an efoil behaves
- promote the Plume brand and its products

# Platforms

Plumefoil Racing should be playable on both PC (Windows and ideally also Mac/Linux) and mobile.

# Technical Stack

My first idea would be to have Plumefoil Racing built in Rust, using Bevy framework/game engine. But I can consider changes if this stack is not the best fit.

# Artistic Direction

Plumefoil Racing is a 3D game. I can provide STEP files for efoil 3D assets. Other assets should be generated (as 3D assets or procedurally whenever relevant).
3D Graphics quality should feel nice without requiring much compute. It's important the game feels reactive and runs well even on a phone.

# Gameplay

Gameplay should feel satisfying. High scores should require some skill. Repeatability is important, the game should not be boring after a quick session.

## Controls

User game controls should include:

- Motor power input
- Rider's stance

## Controller

The game should be compatible with keyboard or gamepad controller on PC. Mobile controls should be tacile joystick and power bar.

# Physics Engine

Using an existing physics engine (if possible), in game behaviour should be physics based (sum of forces = mass \* acceleration).
Water physics (turbulences, waves, etc...) can be simplified so it looks good without being compute heavy.

## efoil components

An efoil is composed of a board, a battery, a mast, a motor and a hydrofoil.
A rider "drives" the efoil balancing/shifting his weight.

Each game component type should have a standard a finite set of characteristics such as weight, buoyancy, lift, drag, etc... (i.e. weight is a constant but may differ between a big board and a small board; buoyancy may be a function of the distance to the water surface; lift and drag should be functions of the speed)
