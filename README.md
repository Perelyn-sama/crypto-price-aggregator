# Cryptocurrency Data Aggregator

## Overview

This Rust project implements a cryptocurrency data aggregator. It focuses on capturing real-time Bitcoin (BTC) price data via WebSocket connections to various cryptocurrency exchanges, processing this data in a distributed client environment, and ensuring secure communication through digital signature verification.

## Features

- **WebSocket Communication**: Connects to cryptocurrency exchanges like Binance for real-time BTC price data.
- **Distributed Client Processing**: Manages multiple client processes to fetch and aggregate data concurrently.
- **Digital Signature Verification**: Ensures the integrity and authenticity of data using ECDSA signatures.

## Prerequisites

Before you begin, ensure you have met the following requirements:
- Rust Programming Language
- Cargo (Rust's package manager)
- Access to cryptocurrency exchange APIs

## Installation

To install the project, follow these steps:

```bash
git clone [your-repo-url]
cd [your-repo-directory]
cargo build
```

## Usage

The project can be operated in two main modes: `cache` and `read`.

### Cache Mode

In cache mode, the client listens to a WebSocket for a given number of times and caches the BTC price data.

```bash
./simple --mode=cache --times=[number_of_times]
```

### Read Mode

In read mode, the program reads from the cached file and displays the data points and the calculated average.

```bash
./simple --mode=read
```

### Running the Aggregator
```bash
cargo run --bin aggregator
```

This command starts the aggregator process along with five client processes.

