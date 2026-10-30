# Contributing to Lumina-Inference

We welcome contributions to the Lumina-Inference project! To ensure a smooth and collaborative development process, please follow these guidelines.

## How to Contribute

1.  **Fork the Repository**: Start by forking the `Lumina-Inference` repository to your GitHub account.
2.  **Clone Your Fork**: Clone your forked repository to your local machine:

    ```bash
    git clone https://github.com/YOUR_USERNAME/Lumina-Inference.git
    cd Lumina-Inference
    ```

3.  **Create a New Branch**: Create a new branch for your feature or bug fix. Use a descriptive name (e.g., `feature/add-model-hot-reloading`, `bugfix/fix-grpc-streaming`).

    ```bash
    git checkout -b feature/your-feature-name
    ```

4.  **Make Your Changes**: Implement your changes, ensuring they adhere to the project's coding standards and best practices. Lumina-Inference is primarily written in Rust, so please follow Rust best practices.

5.  **Test Your Changes**: Run existing tests and add new ones if necessary to cover your changes. Ensure all tests pass.

    ```bash
    cargo test
    cargo fmt --check
    ```

6.  **Commit Your Changes**: Write clear and concise commit messages. Follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

    ```bash
    git commit -m "feat: Implement new model hot-reloading feature"
    ```

7.  **Push to Your Fork**: Push your local branch to your forked repository on GitHub.

    ```bash
    git push origin feature/your-feature-name
    ```

8.  **Create a Pull Request (PR)**: Open a pull request from your forked repository to the **`master`** branch of the original `Lumina-Inference` repository. Provide a detailed description of your changes, including:

    *   What problem does this PR solve?
    *   How were the changes implemented?
    *   Any relevant test results or screenshots.

## Code Style

*   Follow idiomatic Rust practices.
*   Use `rustfmt` for code formatting.
*   Document your code with clear comments and docstrings.

## Reporting Bugs

If you find a bug, please open an issue on the GitHub issue tracker. Provide as much detail as possible, including:

*   A clear and concise description of the bug.
*   Steps to reproduce the behavior.
*   Expected behavior.
*   Screenshots or error messages if applicable.
*   Your environment details (OS, Rust version, installed dependencies).

## Suggesting Enhancements

We welcome suggestions for new features or improvements. Please open an issue on the GitHub issue tracker to discuss your ideas.

## Project status

See [STATUS.md](STATUS.md) and [CHANGELOG.md](CHANGELOG.md).

## Code of Conduct

By contributing to Lumina-Inference, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

Thank you for contributing to Lumina-Inference!
