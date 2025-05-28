# Prefixed Unique Identifier (PUID) Generator

## Overview

This project generates prefixed IDs that can be used as unique identifiers in various applications. Inspired by Stripe, it allows for the creation of type-safe IDs with a specific prefix and a randomly generated 22 character suffix composed of a base62 encoded 128 bit random number.

## Usage

```rs
puid::puid!(UserId = "usr");

fn main() {
    let user_id = UserId::new();
    println!("Generated User ID: {}", user_id); // e.g., usr_45A0IQarTgXyiRM6VQ9YbX
}
```
