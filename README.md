# [AppDataCleaner - AppData cleanup tool for Windows][repo-url]

[![GitHub issues][issues-image]][issues-url]
[![GitHub pull requests][pulls-image]][pulls-url]
[![GitHub stars][stars-image]][stars-url]
[![GitHub forks][forks-image]][forks-url]
[![GitHub downloads][download-image]][download-url]
[![License][license-image]][license-url]
![Repo size][repo-size-image]
<!--[![hits][hits-image]][hits-url1]-->

A free, open-source AppData cleanup tool.

> [!note]
>
> This repository is a fork of the original AppDataCleaner project. It adds an English README translation and may include future feature work.

<details>
<summary><h2>Why this exists</h2></summary>
<p>When software is uninstalled on Windows, files left behind in AppData often remain even after using professional uninstall tools, so this app was created to help clean them up.</p>
<p>The tool is written in Rust, uses egui for the GUI, and was originally generated with the help of ChatGPT.</p>
<p>This fork is kept open and free for contributors who want an English version and future improvements.</p>
</details>

> [!warning]
>
> This project has never been published on GitCode. If you find a copy there, please take a screenshot and keep it as evidence.

## 🖥 System requirements
- Windows 8 or later
- Windows 7 is being tested

## Usage

### 📦 Download the EXE
- [Releases](https://github.com/qbiecom/AppDataCleaner/releases/latest)
- [CI build](https://github.com/qbiecom/AppDataCleaner/actions/workflows/ci.yml)
- [Windows Cleaner (bundled, manually updated)](https://github.com/darkmatter2048/WindowsCleaner)

Choose either download method, then unzip and run the app.

## Star history

<a href="https://star-history.com/#qbiecom/AppDataCleaner&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=qbiecom/AppDataCleaner&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=qbiecom/AppDataCleaner&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=qbiecom/AppDataCleaner&type=Date" />
 </picture>
</a>

### Run the app
> [!caution]
>
> Deletion is irreversible. Please be careful.
- Double-click to run
- Click "Scan now" and the app will scan the AppData folder and show the results
- Choose between "Delete" or "Move" (not implemented yet)

#### About folder descriptions
The app does not ship with any rules by default. Please download a rule set and place it in the project root.

These are the description rules maintained by the original author: [TC999/TC999-subscription](https://github.com/TC999/TC999-subscription)

### Build from source
#### Local build
- Install Rust
- Clone this repository
```
git clone https://github.com/qbiecom/AppDataCleaner.git
```
- Enter the project directory
```
cd AppDataCleaner
```
- Run
```
cargo run
```
- Build
```
cargo build --release
```
- The build output is in `target/release`
#### Or run the CI build directly

## Project structure
- `src`: source code
- `assets`: resources (note: do not delete the font file, or the UI will show squares)
- `Cargo.toml`: dependency manifest

## ✔ TODO
- [x] Whitelist module (prevents accidental deletion and protects important data)
- [x] Folder descriptions
- [ ] Move folder
- [x] Open folder (implemented)
- [ ] Multi-language support (not planned for now)
- [ ] Improve the UI
- [ ] Improve the code
- [ ] Add an application icon
- [x] Project website (implemented with GitHub Pages)
- [ ] More...

See [this discussion](https://github.com/TC999/AppDataCleaner/discussions/7) for more TODO items.

## ✨ Contributing
> [!note]
>
> This repository requires GPG-signed commits. See [GPG signing setup][github-doc-gpg-url].

1. Fork this repository
2. Create a branch named after the feature you are working on; keep each feature in its own code file and import it as a module
3. Commit your changes
4. Open a pull request

See the [contributing guide](CONTRIBUTING.md) for details.

## Acknowledgements
- [TC999](https://github.com/TC999) - original author
- [ChatGPT](https://chatgpt.com/) - code generation
- [egui](https://github.com/emilk/egui) - GUI framework
- [darkmatter2048](https://github.com/darkmatter2048) - CDN provider

### All contributors

[![Contributors](https://contrib.rocks/image?repo=qbiecom/AppDataCleaner)](https://github.com/qbiecom/AppDataCleaner/graphs/contributors)

## 🤝 Support the developer

If you like this project, you can support it here: [project website](http://adc.dyblog.online/donate.html)

![WeChat donation](./readme/wechat.png)

## 📝 License
This project is licensed under the [GPLv3 License](LICENSE).

<!-- Links -->
[issues-url]: https://github.com/qbiecom/AppDataCleaner/issues "Issues"
[issues-image]: https://img.shields.io/github/issues/qbiecom/AppDataCleaner?style=flat-square&logo=github&label=Issues

[pulls-url]: https://github.com/qbiecom/AppDataCleaner/pulls "Pull requests"
[pulls-image]: https://img.shields.io/github/issues-pr-raw/qbiecom/AppDataCleaner?style=flat&logo=github&%3Fcolor%3Dgreen&label=Pull%20requests

[stars-url]: https://github.com/qbiecom/AppDataCleaner/stargazers "Stars"
[stars-image]: https://img.shields.io/github/stars/qbiecom/AppDataCleaner?style=flat-square&logo=github&label=Stars

[forks-url]: https://github.com/qbiecom/AppDataCleaner/fork "Fork"
[forks-image]: https://img.shields.io/github/forks/qbiecom/AppDataCleaner?style=flat-square&logo=github&label=Forks

[discussions-url]: https://github.com/qbiecom/AppDataCleaner/discussions "Discussions"

[hits-url]: https://hits.dwyl.com/ "Views"
[hits-image]: https://custom-icon-badges.demolab.com/endpoint?url=https%3A%2F%2Fhits.dwyl.com%2Fqbiecom%2FAppDataCleaner.json%3Fcolor%3Dgreen&label=Views&logo=graph

[repo-url]: https://github.com/qbiecom/AppDataCleaner "Repository"

[repo-size-image]: https://img.shields.io/github/repo-size/qbiecom/AppDataCleaner?style=flat-square&label=Repo%20size

[download-url]: https://github.com/qbiecom/AppDataCleaner/releases/latest "Download"
[download-image]: https://img.shields.io/github/downloads/qbiecom/AppDataCleaner/total?style=flat-square&logo=github&label=Total%20downloads "Total downloads"

[license-url]: https://github.com/qbiecom/AppDataCleaner/blob/master/LICENSE "License"
[license-image]: https://custom-icon-badges.demolab.com/github/license/qbiecom/AppDataCleaner?style=flat&logo=law&label=License

[github-doc-gpg-url]: https://docs.github.com/zh/authentication/managing-commit-signature-verification/generating-a-new-gpg-key "GPG signing"
