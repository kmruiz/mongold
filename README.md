# mongold poc
[![Build](https://github.com/kmruiz/mongold/actions/workflows/testing.yml/badge.svg)](https://github.com/kmruiz/mongold/actions/workflows/testing.yml)
[![Build](https://github.com/kmruiz/mongold/actions/workflows/security-check.yml/badge.svg)](https://github.com/kmruiz/mongold/actions/workflows/security-check.yml)

Proof of Concept on how to use [tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse multiple language DSL. Each
DSL represents one specific way of querying MongoDB.

⚠️ This is not an official MongoDB product nor it has anything to do with MongoDB. Use at your own discretion. ⚠️

## Packages
* **dialect-interface** exposes a global interface of all possible MQL dialects.
* **dialect-java-driver** implements basic parsing functionality for Java MQL using the official MongoDB Driver.
* **dialect-javascript-driver** implements basic parsing functionality for JavaScript MQL using the official MongoDB Driver.
* **language-server** Exposes language parsing, linter and autocompletion as a LSP enabled server.
* **mongodb-autocompletion** Implements autocompletion based on a running MongoDB Server.
* **mongodb-linting-engine** Lints MQL queries and shows suggestions, warnings and errors depending on a running MongoDB Server.
* **mongodb-query-language** AST of MQL.
* **mongodb-test-fixtures** Internal crate with testing logic that can be reused.
* **mongodb-universe** Represents the current state of the connected MongoDB cluster.