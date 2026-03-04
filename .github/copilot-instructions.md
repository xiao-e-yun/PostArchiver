# GitHub Copilot Instructions

## Overview
This document provides instructions for AI agents working with the Post Archiver codebase. It outlines the architecture, workflows, conventions, and integration points necessary for effective collaboration.

## Architecture
The Post Archiver project is structured to support both Rust and TypeScript components. The Rust backend handles core functionalities, while TypeScript is used for type definitions and client-side interactions.

## Workflows
1. **Installation**: To set up the project, follow these steps:
   - For TypeScript, run:
     ```sh
     npm add -D post-archiver
     ```
   - Import the necessary types in your TypeScript files:
     ```ts
     import type { Post } from "post-archiver"
     ```

2. **Testing the Project**:
   - For Rust, use:
     ```sh
     cargo test --all-features
     ```
   - For TypeScript, execute:
     ```sh
     cargo test -F=typescript
     node gen-types.mjs
     ```
   - This will generate files in the `bindings` directory.

## Conventions
- Follow the established coding standards for both Rust and TypeScript.
- Ensure that all new features are accompanied by tests.

## Integration Points
- The TypeScript bindings are generated from the Rust codebase, allowing for seamless integration between the two languages. Ensure that any changes in Rust are reflected in the TypeScript types.

---