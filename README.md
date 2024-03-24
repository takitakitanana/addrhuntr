# addrhuntr

![Issues](https://img.shields.io/github/issues/takitakitanana/addrhuntr)
![Forks](https://img.shields.io/github/forks/takitakitanana/addrhuntr)
![Stars](https://img.shields.io/github/stars/takitakitanana/addrhuntr)
[![License](https://img.shields.io/github/license/takitakitanana/addrhuntr)](LICENSE)

![image](https://github.com/takitakitanana/addrhuntr/assets/112820741/fae82d53-6d2b-4dda-b6f5-52790c614814)

`addrhuntr` is a specialized tool designed to hunt for specific Ethereum wallet addresses with patterns that are considered rare or desirable.

This tool focuses on generating and matching Ethereum addresses that start with predefined patterns, such as `0xdead`, or those that contain sequences requiring significant computational power to find, like addresses starting with `0x000000`.

## Prerequisites

- You have Docker installed on your system. (https://docs.docker.com/engine/install/)

## Run

1. Clone this repository:
   ```bash
   git clone https://github.com/takitakitanana/addrhuntr.git
   cd addrhuntr
   ```
2. Build the Docker image:
    ```bash
    docker build --no-cache -t addrhuntr .
    ```
3. Start the container (detached mode):
    ```bash
    docker run -d -v ${PWD}/data:/data addrhuntr
    ```
4. Check `data/found.txt` for loot.

## Example

![Animation](https://github.com/takitakitanana/addrhuntr/assets/112820741/d03894b2-f0f8-41eb-b4a0-b2104088b9bf)
![image](https://github.com/takitakitanana/addrhuntr/assets/112820741/7e397b77-c0db-4e05-89f5-6bb56bd06aae)

## Options

![image](https://github.com/takitakitanana/addrhuntr/assets/112820741/7f9280ca-3fcc-4ad9-bfd9-19608be44d41)


- Edit `data/find.txt` with the patterns you are interested in.

`0x1234` will match addresses starting with *1234*.  
`0xdead...dead` will match addresses starting with *dead* and ending in *dead*.

- Adding `-u` (USER ID) and `-d` (Discord Webhook URL) flags.

![image](https://github.com/takitakitanana/addrhuntr/assets/112820741/a66702b2-fbf3-49cc-a296-08123cfb7177)  
(Above screenshot was made for demo purposes. To avoid spam - currently it is set to ping you on discord only for addresses starting with `0x00000000`.)

Discord Documentation:  
`-u` ->
https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID  
`-d` -> https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks

## To-Do

Here are some enhancements that are planned for future releases of `addrhuntr`:

- **Unified Timestamps**: Ensure the timestamp for when an address is found and when it is written to `found.txt` uses the same variable, providing precise synchronization.

- **Bare-Metal Instructions**: Add detailed instructions for compiling and running the Rust binary directly on the system without Docker, allowing users more flexibility in how they run `addrhuntr`.

- **Customizable Discord Mention**: Implement an argument (`argv`) that allows users to specify the keyword for Discord pings, moving away from the current hardcoded value.

- **Multi-Threading Support**: Introduce multi-threading capabilities to enhance performance and utilize computational resources more efficiently.

- **Code Refactor**: Refactor the codebase for better maintainability, readability, and performance optimization.

- **Implement Multi-Stage Docker Build**: Transition to a multi-stage Docker build process to reduce the final image size by compiling the application in a Rust build environment and then moving the compiled binary to a smaller base image for runtime.

- **Verbose Statistics Feature**: Add a command-line argument (`--stats` and `-s`) to enable more detailed statistics during runtime for users who require in-depth information about the address generation process.

- **Docker Image Hosting**: Publish the `addrhuntr` Docker image to a container registry such as Docker Hub, tagged with versions and `latest` to simplify deployment and distribution.

- **Blockchain Support Expansion**: Extend functionality to generate and analyze addresses for Bitcoin and other blockchains, enhancing the utility of `addrhuntr` across different cryptocurrency ecosystems.

Contributions to address these points are welcome, appreciate any input or pull requests from the community to help improve `addrhuntr`.
