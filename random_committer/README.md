# Random Committer 🎲

Welcome to Random Committer, a magical program that adds a sprinkle of chaos to your GitHub repository by making random commits at random hour intervals. Think of it as a digital jester, keeping your repository lively and unpredictable!

## 🌟 Features
- Makes a random number of commits (between 1 and 48) to a designated repository at random hourly intervals.
- Commits a randomly generated number to a file in your repository.
- Automatically retries failed commits at the next hour.
- Powered by Rust, Git2, and a pinch of fun!

## 🚀 Getting Started

### Prerequisites
- Rust Programming Language (v1.40 or above)
- Git2 Library
- Chrono Library
- Crypto Library
- Rand Library

### Installation
1. Clone this repository.
2. Navigate to the project folder.
3. Run `cargo build` to compile the code.

### Configuration
Before running, you'll need to modify the following placeholders in the code:
- `"PATH/TO/YOUR/PRIVATE/REPO"`: Replace with the path to your private GitHub repository.
- `"YOUR NAME"` and `"YOUR EMAIL ADDRESS"`: Replace with your name and email address.
- `"YOUR USER NAME"` and `"YOUR GITHUB ACCESS TOKEN, GET IT HERE: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens"`: Replace with your GitHub username and personal access token.

### Usage
Run `cargo run` from the command line, and watch as your repository becomes the home of whimsical and unpredictable commits.

## 🎭 Why Use Random Committer?
This tool was designed with whimsy in mind. Maybe you want to keep your GitHub streak alive, test automation tools, or simply add a touch of randomness to your day. Whatever your reasons, Random Committer is here to make code a little more fun!

## 🛑 Caution
- **Use with care!** This program makes real commits to your GitHub repository and can potentially alter your project's history.
- **Do not use on critical repositories.** Ideal for experiments, personal projects, or repositories where a touch of chaos is welcomed.

## 📜 License
This project is licensed under the MIT License. See the [LICENSE.md](https://github.com/sebsadface/fun_stuff/blob/main/LICENSE) file for details.

## 🎉 Have Fun!
Embrace the unpredictability and enjoy the whimsical dance of randomness. Happy committing! 🚀

---

*This README was written with joy and a smile. For any concerns, issues, or suggestions, please open an issue on GitHub or reach out to the author.*
