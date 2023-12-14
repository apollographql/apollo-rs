# Contributing to `apollo-rs`

> `apollo-rs` is a project by [Apollo GraphQL] and is not currently ready for
> external feature contributors, though some documentation contributions may be
> accepted.

## Prerequisites

`apollo-rs` is written in [Rust]. In order to contribute, you'll need to have
Rust installed. To install Rust, visit [https://www.rust-lang.org/tools/install].

Rust has a build tool and package manager called [`cargo`] that you'll use to
interact with the code.

## Workflows

To compile the project:
```bash
cargo build
```

To run tests:
```bash
cargo test --workspace
```

To generate parser API after changes to `ungrammar` file:
```bash
cargo xtask codegen
```

To run parser and lexer tests _and_ let the testing framework make changes to the incorrect files on a unix system:
```bash
env UPDATE_EXPECT=1 cargo test --package apollo-parser
```
or the same in powershell:
```bash
$env:UPDATE_EXPECT=1; cargo test --package apollo-parser
```

To run compiler tests _and_ let the testing framework make changes to the incorrect files on a unix system:

```bash
env UPDATE_EXPECT=1 cargo test --package apollo-compiler
```
or the same in powershell:
```bash
$env:UPDATE_EXPECT=1; cargo test --package apollo-compiler
```

You can also run individual test files from the `test_data` directory in 
`apollo-compiler` crate. The output will be the current file's validation diagnostics:
```bash
cargo run --example validate /path/to/test/file
```

To format your code:
```bash
rustup component add rustfmt
cargo fmt --all
```

To lint your code:
```bash
rustup component add clippy
cargo xtask lint
```

### IDEs

We primarily use [VS Code] along with [rust-analyzer] when developing.
`rust-analyzer` can also be used with [other IDEs] if you are more
familiar with something else.

[Apollo GraphQL]: https://www.apollographql.com
[Rust]: https://www.rust-lang.org/
[`cargo`]: https://doc.rust-lang.org/cargo/index.html
[https://www.rust-lang.org/tools/install]: https://www.rust-lang.org/tools/install
[VS Code]: https://code.visualstudio.com/
[rust-analyzer]: https://rust-analyzer.github.io/manual.html
[other IDEs]: https://rust-analyzer.github.io/manual.html#installation

## How to contribute

### Using issues

We work largely in public using GitHub [issues] to track work. To make sure contributions are aligned with the project's goals, keep the following issue etiquette in mind:

* [Open an issue] for your contribution. If there is already an issue open, please ask if anyone is working on it or let us know you plan on working on it. This will let us know what to expect, help us to prioritize reviews, and ensure there is no duplication of work.
* Use issue templates! These templates have been created to help minimize back-and-forth between creators and the `apollo-rs` team. They include the necessary information to help the team triage your issue or question, as well as automatically applying the appropriate labels.
* Issues with the `triage` label still applied have not yet been reviewed by the `apollo-rs` team, and there are no guarantees that PRs fixing an untriaged issue will be accepted. It's best to wait for issues to be triaged before beginning work.

### Submitting a Pull Request

Pull requests (PRs) should only be opened after discussion and consensus has been reached in a related issue, and you have communicated your intentions to create a PR with the `apollo-rs` team.

* When creating a PR, make sure to link it to an issue or use the `Fixes #123` syntax to make sure others know which issue(s) your PR is trying to address and to help us automatically close resolved issues.
* Include a helpful description. It is important to provide context to reviewers that show _how_ your PR addresses an issue and any questions you still have unanswered, or portions of the code you think deserve some extra attention.
* If your work is still in-progress and you're opening a PR to get early feedback, let us know by opening it as a draft PR and adding `wip:` prefix in the PR title.
* Add tests for any logic changes in your code, especially if you are fixing a bug. Your PR should have no failing tests before merging. Please let us know if you need help writing tests, there are still some portions of the `apollo-rs` codebase that do not have established testing patterns.

[issues]: https://github.com/apollographql/rover/issues
[Open an issue]: https://github.com/apollographql/rover/issues/new/choose


## Code of conduct

The project has a [Code of Conduct] that *all* contributors are expected to
follow. This code describes the *minimum* behavior expectations for all
contributors.

As a contributor, how you choose to act and interact towards your fellow
contributors, as well as to the community, will reflect back not only on
yourself but on the project as a whole. The Code of Conduct is designed and
intended, above all else, to help establish a culture within the project that
allows anyone and everyone who wants to contribute to feel safe doing so.

Should any individual act in any way that is considered in violation of the
[Code of Conduct], corrective actions will be taken. It is possible, however,
for any individual to *act* in such a manner that is not in violation of the
strict letter of the Code of Conduct guidelines while still going completely
against the spirit of what that Code is intended to accomplish.

Open, diverse, and inclusive communities live and die on the basis of trust.
Contributors can disagree with one another so long as they trust that those
disagreements are in good faith and everyone is working towards a common goal.

## Bad actors
All contributors tacitly agree to abide by both the letter and spirit of the
[Code of Conduct]. Failure, or unwillingness, to do so will result in
contributions being respectfully declined.

A *bad actor* is someone who repeatedly violates the *spirit* of the Code of
Conduct through consistent failure to self-regulate the way in which they
interact with other contributors in the project. In doing so, bad actors
alienate other contributors, discourage collaboration, and generally reflect
poorly on the project as a whole.

Being a bad actor may be intentional or unintentional. Typically, unintentional
bad behavior can be easily corrected by being quick to apologize and correct
course *even if you are not entirely convinced you need to*. Giving other
contributors the benefit of the doubt and having a sincere willingness to admit
that you *might* be wrong is critical for any successful open collaboration.

Don't be a bad actor.

[Code of Conduct]: https://github.com/apollographql/.github/blob/main/CODE_OF_CONDUCT.md

