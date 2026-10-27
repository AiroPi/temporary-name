# Awful code but great idea (maybe)

Want to make a tool that:
- talks to Docker socket to get running containers images
- runs `syft` against these images to extract sboms
- runs `grype` against these sboms to extract CVE
- shows this in a dashboard
- send alerts if some CVE are worth

I'm using Rust because I don't know Rust.

I may use AI for this project because I don't know how to use AI.

