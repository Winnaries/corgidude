# :dog: CorgiDude + ⚙️ Rust = :heart:

A general Rust implementation and example for CorgiDude board by [AiDude.io](https://aidude.io).

### Board Crate

High-level and low-level abstractions for the board and Kendryte K210 to be used repeatly across project. The goal 
for this crate is to become an official `corgidude-hal` crate of embedded Rust. 

### RGB 

Simple GPIO and FPIOA testing via RGB led. 

### Streaming

Retrieving image from OV2640 via DVP peripheral and displays it on 240x240 screen using DMAC and SPI0.
