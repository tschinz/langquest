# Changelog

All notable changes to this project will be documented in this file.

## [0.2.0] - 2026-09-10

### 🚀 Features

- *(scripts)* One-call install scripts for latest release ([07d7716](https://github.com/tschinz/lq/commit/07d7716f27ab8830743747cd2f165c32332496d4) - axel.amand)
- *(doc)* Install latest one-line link pointing to main branch ([07dc1b0](https://github.com/tschinz/lq/commit/07dc1b03601964c01582446cf48e9102cebb4c10) - axel.amand)
- *(cli)* Add -k /--keys ([420e83d](https://github.com/tschinz/lq/commit/420e83dcbf2f980a6cc3ffee500bc21a27097e58) - axel.amand)
- *(ci)* Linux musl release ([a222917](https://github.com/tschinz/lq/commit/a222917cf0c868c7def47e0bc5f650064ecb3a46) - Axam)
- *(crypto)* Allow arbitrary key sizes ([28441b9](https://github.com/tschinz/lq/commit/28441b948803b8fbca7d91b07d909c0513128ec7) - mel-64)
- *(save-counter)* Add save-counter feature ([1e34a87](https://github.com/tschinz/lq/commit/1e34a87d8b7d6451bc918877393adac6d5af02cd) - mel-64)
- *(rust-project)* Autocreate .rust-project on rust exercise entry ([1417242](https://github.com/tschinz/lq/commit/141724267f4896e514d6528b96da61f9f5d2b719) - mel-64)
- *(cli)* Add -r shorthand flag ([ef28458](https://github.com/tschinz/lq/commit/ef28458c17683efcf869623942d9aa13481c6352) - mel-64)
- *(plantuml)* Remove plantuml.jar from the assests. It's up to the users to install plantuml and provide the path through env variable ([759aff1](https://github.com/tschinz/lq/commit/759aff12aaf4a6aec6c5bf124e32338c89c5f966) - Borgeat Rémy)
- *(just)* Add plantuml toolchain installation in the just setup recipe ([7f1fe82](https://github.com/tschinz/lq/commit/7f1fe82aa49b163e7130ff9ec3d81fb9abe0e1d0) - zas)
- *(tooolchain)* Check toolchain setup at startup. ([71f09f5](https://github.com/tschinz/lq/commit/71f09f5deb3abc8659ae460f295c1a1159153c34) - zas)
- *(grading)* Add grading flag which does not validate identity nor write to progress file ([d757cae](https://github.com/tschinz/lq/commit/d757caeb4b30b32239867c42cc6fb5250f4a5d35) - mel-64)
- Add keybind for opening dir in editor ([edb5cca](https://github.com/tschinz/lq/commit/edb5ccaf89199bb71393bdbaf26bb370856548ea) - mel-64)
- Add config polling option ([5357d72](https://github.com/tschinz/lq/commit/5357d725d142744f7af28b23a54bdfcc94e2b719) - mel-64)
- *(rust/cargo)* Add cargo compatibility ([68cac5e](https://github.com/tschinz/lq/commit/68cac5ec423a9ea13eea7a377073f2180e334395) - mel-64)
- *(lq.toml)* Use defaults if not set ([4d7b671](https://github.com/tschinz/lq/commit/4d7b671df737642c95773e51f4a53434daa6701b) - mel-64)
- *(plantuml)* [**breaking**] Add plantuml regex grading ([f1a10e5](https://github.com/tschinz/lq/commit/f1a10e5ee2ef3b4b0a2925178505f875dbbd62ea) - mel-64)
- *(plantuml)* Match reges literals and whitepace ignores literals ([4882b4f](https://github.com/tschinz/lq/commit/4882b4fde69222214c7026f98bd5b1e463dd680f) - zas)
- Allow overwriting the source file in 02-task.md ([dd197db](https://github.com/tschinz/lq/commit/dd197db9becb80741aa479290cf550c386039929) - mel-64)

### 🐛 Bug Fixes

- *(scripts)* Fixed to latest release download ([1587b2c](https://github.com/tschinz/lq/commit/1587b2c607a70724926c657b3d74b3b7d3331b5f) - axel.amand)
- *(ci)* Passes secrets in release build ([b5fb286](https://github.com/tschinz/lq/commit/b5fb286c1f28ce0a07bb8defec8a992e147e9388) - axel.amand)
- *(build)* Don't rebuild due to .env{ , .template} if it isn't present ([ae4b76f](https://github.com/tschinz/lq/commit/ae4b76f8953fc4f58c1563bc3a01ed915d6e73ed) - mel-64)
- *(save-counter)* Show overall times saved in CLI stats output ([3110333](https://github.com/tschinz/lq/commit/31103334d6119cb7abe12f37e23426f89898f3e1) - mel-64)
- *(rust-tests)* Fix rust tests including errors like stacktraces counting an incorrect total ([094add5](https://github.com/tschinz/lq/commit/094add57a9bfd3a918cec16c59a7b15997430150) - mel-64)
- *(rust-score-parsing)* Use clean test output for student score parsing ([07d6178](https://github.com/tschinz/lq/commit/07d61785e1e7ddf358cc86bd87d7705da14aa5a6) - mel-64)
- *(output)* Use stdout instead of stderr for TUI ([56bde48](https://github.com/tschinz/lq/commit/56bde48b00962896995e05baf0bbdca95284b98f) - mel-64)
- *(build)* Correct MSRV ([307e220](https://github.com/tschinz/lq/commit/307e22083b48e68ff8377841eb832373dc8a8231) - mel-64)
- *(ci)* Fix rustfmt ([f468be4](https://github.com/tschinz/lq/commit/f468be46bae3a08a4589278f88ee68c63a0bca2c) - Borgeat Rémy)
- *(test)* Add example opf missing plantuml config in the example ([5907a61](https://github.com/tschinz/lq/commit/5907a61695c7a435d1b0e609ccdb95680a30623a) - zas)
- *(startup)* Do not generate and open if we start on a plantuml exercises ([48f36df](https://github.com/tschinz/lq/commit/48f36df8959d61df7eda5bd6933c60c7e089f22d) - zas)
- *(eprint)* Tui own stdout therefore pretui use stderr ([4dfd235](https://github.com/tschinz/lq/commit/4dfd235c9c257fcdfd80c5aa5de0cdadf22bedb0) - zas)
- *(TUI)* Remove stray debug println ([3c05313](https://github.com/tschinz/lq/commit/3c0531398df4d48a4c8957fa460f936114a857b2) - melody)
- *(debug_output)* Cap debug output to MAX_OUTPUT_LINES ([2f5bd25](https://github.com/tschinz/lq/commit/2f5bd25d34928667cbbfdd6a82ae79a7d18c5fdc) - mel-64)
- *(grading)* Lock behind feature flag, disallowing students to cheat ([696a491](https://github.com/tschinz/lq/commit/696a491675f44124de02197a961e350791cf7630) - mel-64)
- *(cargo)* Fix cargo build cache and more explicit error-handling ([814c261](https://github.com/tschinz/lq/commit/814c261646ae7a7afdf5d8cacd4911d06aeb8e0e) - mel-64)
- *(just)* Since the addition of cargo exercise we need to precise which binary to install ([41001d0](https://github.com/tschinz/lq/commit/41001d0ef02ff9a9e38047d8862c5c5a12cbbfaa) - zas)
- *(sample-repo)* Add missing file content ([f3a731f](https://github.com/tschinz/lq/commit/f3a731fe2a1e98c5f57e12224808894c560565dd) - mel-64)
- *(child-hanging)* Fix child hanging due to full pipe buffer ([20bb9d3](https://github.com/tschinz/lq/commit/20bb9d3cfa3106027c0cb8a72e03bf9d5280d362) - mel-64)
- *(cargo-toml-watcher)* Fix cargo.toml not triggering re-validation ([6ce63d8](https://github.com/tschinz/lq/commit/6ce63d800b6fe4a95cfe908498bc5967c1dfdae0) - mel-64)
- *(runer)* Reload files struct was missing ([50df709](https://github.com/tschinz/lq/commit/50df709fc5879517717d237194d252e34cd4c32c) - zas)

### 💼 Other

- Merge pull request #15 from tschinz/feat/one-line-release-install

HEI | One-line release install with hidden keys ([8cfee9b](https://github.com/tschinz/lq/commit/8cfee9b7ea3b46632d0e2c5ba9eaa78fae83b046) - Axel Amand)
- Rename 'keys' argument short flag from 'v' to 'k'fix(app): -k wrongly parsed ([386622c](https://github.com/tschinz/lq/commit/386622caa01b0eb4950cb5430c98a6d9b6d41e6a) - Axel Amand)
- Merge pull request #17 from tschinz/ci/linux-musl

Linux MUSL build ([105d920](https://github.com/tschinz/lq/commit/105d920a09c9071dee868160ece17ec0f0ef7d1d) - Qarmma)
- Merge branch 'main' into feat/grading ([1d4aad1](https://github.com/tschinz/lq/commit/1d4aad151f7378eeb9f47edc3d1805f134377177) - tschinz)
- Merge pull request #49 from mel-64/chore/update-workflows

chore(ci): update workflow tags ([2e044c0](https://github.com/tschinz/lq/commit/2e044c02b0f8e98ecd4f1ca87a3ab6977911b16d) - melody)
- Merge branch 'main' into feat/source-file-overwrite ([72dd18d](https://github.com/tschinz/lq/commit/72dd18df9cab1d4296c93fd3bb04da7cefe0146a) - mel-64)
- Merge commit 'pullrequests/mel-64/feat/source-file-overwrite' ([be048a9](https://github.com/tschinz/lq/commit/be048a997af24fc9ed88ea21d87571b7ae502247) - zas)

### 🚜 Refactoring

- *(app.rs)* Rename maybe_create_rust_project to maybe_create_rust_project_json ([8b276e0](https://github.com/tschinz/lq/commit/8b276e03127495ffb452bcec0e1a423a51bb8a6b) - mel-64)
- *(plantuml)* Inline puml comment stripping regex ([4890cc7](https://github.com/tschinz/lq/commit/4890cc70a19af8556424b6097ff318a9bf162bba) - mel-64)

### 📚 Documentation

- *(arb. key sizes)* Add documentation to .env.template ([9ba57c2](https://github.com/tschinz/lq/commit/9ba57c2d4f746dc15fa45446904590ba49ebf4e7) - mel-64)
- *(testing)* Add test dependency section to README.md ([5dd01bd](https://github.com/tschinz/lq/commit/5dd01bd44ade76d3fc52ea3c2e63ddac16ddd710) - mel-64)
- *(readme)* Update stats file output docs ([da949d6](https://github.com/tschinz/lq/commit/da949d62f2d63181dfef2214760b7b1df3588746) - mel-64)
- *(readme)* Add some plantuml information ([b35885f](https://github.com/tschinz/lq/commit/b35885fe0988cefeb87e0c5097ac88b1a332cf29) - zas)
- *(grading-mode)* Add documentation section to README.md ([f58b955](https://github.com/tschinz/lq/commit/f58b955384b8a780a55e7b96b81dd1a910906ef1) - mel-64)
- *(readme)* Update various README.md docs ([5b1ff7c](https://github.com/tschinz/lq/commit/5b1ff7cba00a9cfba54b7461ed9aeb9a1e0ffbb1) - mel-64)
- *(keybinds)* Add keybind docs to README.md ([1a262c8](https://github.com/tschinz/lq/commit/1a262c8d9774838bb334309f26ec9b19e1c9803c) - mel-64)
- *(polling watcher)* Add docs to README.md ([1c99875](https://github.com/tschinz/lq/commit/1c998758b67a20c6b35875535b2b5b6fe9c66c24) - mel-64)
- *(cargo)* Document cargo examples ([c24ecf0](https://github.com/tschinz/lq/commit/c24ecf056efefb928c135eef38d0d4f409679370) - mel-64)

### 🧪 Testing

- *(cargo)* Add default cmd_cargo to rust section ([b1f271b](https://github.com/tschinz/lq/commit/b1f271bede4dc382267b7dfaf4e70b57b7063739) - mel-64)
- *(cargo)* Add cargo test project ([baafa3b](https://github.com/tschinz/lq/commit/baafa3b8ca0f5b35dba3ea1abd5ed64e87ed8c66) - mel-64)

### ⚙️ Miscellaneous Tasks

- *(cargo.toml)* Add rust-version ([6ae13a0](https://github.com/tschinz/lq/commit/6ae13a0639c0cfb85ee4f9d3fbdba93de1a27347) - mel-64)
- Clippy fixes ([c2e6c6e](https://github.com/tschinz/lq/commit/c2e6c6e8c845b092ad84538907a6ebd4c5041259) - mel-64)
- Bump msrv to 1.88.0 ([c431a7e](https://github.com/tschinz/lq/commit/c431a7eaa876f732238f31e8d14a15c8f62f3be8) - mel-64)
- Add Cargo.lock ([5f4d060](https://github.com/tschinz/lq/commit/5f4d060c6bfc9dce52b3d324e8c16739d637edfa) - mel-64)
- Update .gitignore ([44bf8a8](https://github.com/tschinz/lq/commit/44bf8a8a1c47e4cf2e6370845060e3c400a20d04) - mel-64)
- *(ci)* Update workflow tags ([963ad08](https://github.com/tschinz/lq/commit/963ad082639a9b34ac31e474f389902034518d19) - mel-64)
- Fmt fixes ([c6fa4da](https://github.com/tschinz/lq/commit/c6fa4dac33476d3b357e092b02ea198ee6a99723) - mel-64)


**Full Changelog**: [v0.1.0...0.2.0](https://github.com/tschinz/lq/compare/v0.1.0...0.2.0)

## [0.1.0] - 2026-08-04

### 🚀 Features

- *(ripes)* Add ripes executable for win, mac, linux ([8b90898](https://github.com/tschinz/lq/commit/8b90898a2fb76a773d678f7568e61489b93f33aa) - zas)
- *(app)* Add basic app functionalities ([149a0a2](https://github.com/tschinz/lq/commit/149a0a2e57807e08e378d5a1b738f9c77cee6e86) - zas)
- *(ui)* Add ratatui ui ([7035c73](https://github.com/tschinz/lq/commit/7035c7369cdbf58b5ea76a3a6cc5c97cf73a48ca) - zas)
- *(cache)* Add caching, small statusbar changes ([6688c6d](https://github.com/tschinz/lq/commit/6688c6dc67d04b4f0e595437f8a2a8803cc50e4a) - zas)
- *(scoll)* Add content_height and viewport_height ([e9ebfd2](https://github.com/tschinz/lq/commit/e9ebfd28f165e1ba12fd4ee972827c7cc0134196) - zas)
- *(cpp)* Support c++ as langunages through catch2 with hello world example ([7b7134c](https://github.com/tschinz/lq/commit/7b7134c3c2ed4b3a14270cdcdb24d28cd444df65) - zas)
- *(debug)* Print the main logs under the debug pannel ([3acf946](https://github.com/tschinz/lq/commit/3acf946cc60b9d7bd0dadbd2db3a116a61f9dbaa) - Borgeat Rémy)
- *(modules)* Search exercices recursively ([497c4b9](https://github.com/tschinz/lq/commit/497c4b918d69bc0c446e9e2c709ca8a4e33e413c) - Borgeat Rémy)
- *(exercices)* Add exercices in subfolders ([186b527](https://github.com/tschinz/lq/commit/186b5276fea44b5331616160f886f69af3241d3e) - Borgeat Rémy)
- Register map declared only once ([f6c783e](https://github.com/tschinz/lq/commit/f6c783e77a15c6a7c0a6d1055e08be48b0b84b8a) - Axam)
- Add(table) : Support markdown table. The markdown tables are now
rendered in the terminal ([5900be4](https://github.com/tschinz/lq/commit/5900be4a23e80ed357f9c7ca1fa3864d78eaad01) - Borgeat Rémy)
- *(hints)* Support multilines hint. ([804f69f](https://github.com/tschinz/lq/commit/804f69f5e3aee8f73ba5a1ceb36b59bb63b15c7b) - Borgeat Rémy)
- Feat(hint): Minor modification to hint rendering. A line-break and
tabulation is automatically added. ([e36d369](https://github.com/tschinz/lq/commit/e36d36967e98c6fe73e23abd281a32233244a45b) - Borgeat Rémy)
- Feat(hint): Pre-wrap hint text to keep wrapped lines aligned with
4-space indent ([66994ad](https://github.com/tschinz/lq/commit/66994add5880de24081f0200a0b0e56f3f78ff20) - Borgeat Rémy)
- Register map declared only once ([fdcaedd](https://github.com/tschinz/lq/commit/fdcaedd26827fcf59e7e40860fc4960751ca3a28) - Axam)
- Case insensitive registers name check ([185a01a](https://github.com/tschinz/lq/commit/185a01a310392a6be5af2f27c99141b33cf99248) - axel.amand)
- *(riscv)* Add default timeout of 5 seconds ([f75531e](https://github.com/tschinz/lq/commit/f75531e95e08af127fedc68dcd0c59a3f1eac950) - axel.amand)
- *(overview)* Refactor overview menu ([c70418a](https://github.com/tschinz/lq/commit/c70418ab9c06a02cb55177bfda7a2a1f18b3c258) - Borgeat Rémy)
- *(riscv)* Cycles count in ripes output printed to run results ([0ff8244](https://github.com/tschinz/lq/commit/0ff82447655b35275a7f413af543fb8b4683e2c8) - axel.amand)
- *(riscv)* Output cycles not found if missing from ripes output ([de20a0f](https://github.com/tschinz/lq/commit/de20a0fe1a442758719b4e54d12fc7a1750a8dc8) - axel.amand)
- *(riscv)* M extension activated by default ([cdb78c1](https://github.com/tschinz/lq/commit/cdb78c1b299396ed7cf9877b86fc89145ba53d54) - axel.amand)
- *(progressbar)* Refactor progress bar to display exercices status ([9d0f936](https://github.com/tschinz/lq/commit/9d0f936a4631792879bb28ff9aa4d8d2aa5ef5a0) - Borgeat Rémy)
- *(editor)* Window now open files in deault editor ([409e35c](https://github.com/tschinz/lq/commit/409e35ce66432bdb00dd5d90c97592e105139ce2) - Axam)
- Feat(stats): Add number of hints shown and the max hint show in the
config.toml ([d74d09c](https://github.com/tschinz/lq/commit/d74d09c4d922e0d0fedeec8fa8bce66e9bb1ddd1) - Borgeat Rémy)
- *(stats)* Add a statistics command ([13fb31e](https://github.com/tschinz/lq/commit/13fb31e33052b26710ec3b5feed2c2ff1144ea0a) - Borgeat Rémy)
- Update UI before verifying ([b2c2c0c](https://github.com/tschinz/lq/commit/b2c2c0c9b16456a3c5e89e5c46ac60b35a45e8b0) - axel.amand)
- Async verify ([0f3532c](https://github.com/tschinz/lq/commit/0f3532c67c5c0a955103ab18248d7b26811d9ed2) - axel.amand)
- Update UI before verifying ([8fa0685](https://github.com/tschinz/lq/commit/8fa06850a4cb0087b75383907da91a104a99fe13) - axel.amand)
- Async verify ([6dcc763](https://github.com/tschinz/lq/commit/6dcc7630a13f46c5a964d367e225325cd8180651) - axel.amand)
- *(tamperproof)* Split and encrypt config. Read only ba everyone write only be verified gh-cli user ([51496b1](https://github.com/tschinz/lq/commit/51496b1e9e87d74709c9abe3a89517ab0df5ed95) - zas)
- *(seal)* Add feature for working with the unsealed and sealed solutions. Possibility to seal solutions ([dd73fee](https://github.com/tschinz/lq/commit/dd73fee1d12ef7df1739c22b3755910203b7f678) - zas)
- *(solutions seen)* Solution seen counter won’t update once exercise passed ([d028d05](https://github.com/tschinz/lq/commit/d028d057f6bc6a25c1ee14c60e060a103521065e) - zas)
- *(hints)* Increment hints only if exercise is not passed ([08bdc15](https://github.com/tschinz/lq/commit/08bdc153e2768dc208902b403752738bca69fa50) - zas)
- *(export)* Add result.toml export ([b903006](https://github.com/tschinz/lq/commit/b9030069bf87141fdd1cc6839fb13a3efbc7d156) - zas)
- *(unittests)* Add unit test to the stats ([e23fde6](https://github.com/tschinz/lq/commit/e23fde65ba15a47ab6fdbd73bae8f12c90a835a0) - zas)
- *(plantuml)* Add possibilitie to create plkantuml exercises ([285d713](https://github.com/tschinz/lq/commit/285d713926a33ebeb1df51898370122e7c22704d) - zas)
- *(config)* Add ide settings in config. All files use this not the system default. ([f8efa50](https://github.com/tschinz/lq/commit/f8efa508e3e7d3441f01cff2c1ea5974093eb263) - zas)
- *(crypto)* Passing keys through build script from .env ([d0ec10e](https://github.com/tschinz/lq/commit/d0ec10e14201c2ea11c617611a07310a5bace241) - Axam)
- *(build)* No auto-copy from .env.example if .env missing ([d1e5310](https://github.com/tschinz/lq/commit/d1e53105132d0b69588ad9416d512ce198688296) - axel.amand)
- *(ci)* Placeholders env variables if not set by CI secrets ([f2a8aad](https://github.com/tschinz/lq/commit/f2a8aad7e7a261ceebb5f5f2b8e21f10152c7f7d) - axel.amand)
- *(build)* Log source for each key in build process ([cca4162](https://github.com/tschinz/lq/commit/cca4162cfc634481c468fa91c6f498421bb51a3e) - axel.amand)

### 🐛 Bug Fixes

- *(img)* Image support dark and white theme ([3883135](https://github.com/tschinz/lq/commit/38831350dadfd936ab187c078c18a05d7e3f0207) - zas)
- *(just)* Problem with fmt and rust editions ([13bc7f2](https://github.com/tschinz/lq/commit/13bc7f2e3d1a4b18730f67e2aaf8b15adc0e9c4d) - zas)
- *(asm)* # vs ; comment ([a5b89c9](https://github.com/tschinz/lq/commit/a5b89c9d2845d0abd7189ba4ecd5fc7efcbfbeab) - zas)
- *(logo)* Add title and color ([631c15f](https://github.com/tschinz/lq/commit/631c15f0bd56f7a1fd8b02f94001f94396f41221) - zas)
- *(readme)* Typos ([ffb55a0](https://github.com/tschinz/lq/commit/ffb55a023f2c8519850fd2c5e97c1758672da6ee) - zas)
- *(just)* Github link and setup vs install recipe ([762d572](https://github.com/tschinz/lq/commit/762d5721baab1be90d28c56f2dd3898864d43efd) - zas)
- *(ai)* — vs - ([a500f61](https://github.com/tschinz/lq/commit/a500f61c2fc34c61352d6e83b4a66cb37b4738b1) - zas)
- *(check)* Do not show the keywords in output window add check line ([f5ea5a4](https://github.com/tschinz/lq/commit/f5ea5a49d8bb8f9fc65af6be07ee2de5a7be30e5) - zas)
- *(tui)* Windows compatibility ([81ef85c](https://github.com/tschinz/lq/commit/81ef85c4b310a0f9e09a212bc34a400737c483db) - zas)
- Fix(just) shell settings ([f13546f](https://github.com/tschinz/lq/commit/f13546ff50f16fa4fa051bdfa172b4b8ad33b57c) - zas)
- Fix(ui) : Fix laggy UI on Windows ([9da40f1](https://github.com/tschinz/lq/commit/9da40f1eac3519b3a6d4b84c695bf2cff3b142c8) - Borgeat Rémy)
- Fix(ci) : Fix clippy and failling test ([e58447a](https://github.com/tschinz/lq/commit/e58447aa2fb93240bdae870fd00ed19939c92773) - Borgeat Rémy)
- Fix(ci) : Fix rustfmt issues ([6a1815d](https://github.com/tschinz/lq/commit/6a1815d83912a64bbdab2f27294be9e5bb3bb938) - Borgeat Rémy)
- ABI expected regs transformed to xNN variants to match Ripes log output ([cb6c766](https://github.com/tschinz/lq/commit/cb6c766fba9658eb74f5fe80411eacb3b72f1c63) - Axam)
- Rustfmt ([e338022](https://github.com/tschinz/lq/commit/e3380228009fd3fa6bfb4fdb64dca62a26de83e0) - Axam)
- *(ci)* Avoid text file busy error ([a5db2c9](https://github.com/tschinz/lq/commit/a5db2c9d56dfcb3a21954d5b876336eb9b1f7370) - Borgeat Rémy)
- ABI expected regs transformed to xNN variants to match Ripes log output ([7494bf8](https://github.com/tschinz/lq/commit/7494bf8ce4554e141bd754d5a2339d9a1404f1c7) - Axam)
- Rustfmt ([ea4dae2](https://github.com/tschinz/lq/commit/ea4dae2867c74c601f149e1e9e1f3b1856f5e589) - Axam)
- Rustfmt ([4d8dd7d](https://github.com/tschinz/lq/commit/4d8dd7d6bb6755208d269eab5d9431c9e9d70c5d) - axel.amand)
- Fmt ([e4f4b46](https://github.com/tschinz/lq/commit/e4f4b467b74df22af43eece300df30a9c90fca9e) - axel.amand)
- *(clippy)* Useless format ([e2d3983](https://github.com/tschinz/lq/commit/e2d39833f3f4438d1edfc5d8ca73477a2940ec9d) - axel.amand)
- *(ui)* Small bugs fixes in overview panel ([09af818](https://github.com/tschinz/lq/commit/09af81875fe6b918b726a970b421dcfe9063fe51) - Borgeat Rémy)
- *(clippy)* Fix clippy errors ([5494727](https://github.com/tschinz/lq/commit/5494727da4229a7ec1a5dd807931d31b17d405e7) - Borgeat Rémy)
- Fix(hint) : Hints were not shown correctly in some cases ([c4ebce3](https://github.com/tschinz/lq/commit/c4ebce398fb9f5d21baeac7ba73a9575a09b6fe0) - Borgeat Rémy)
- *(riscv)* Ripes on Linux outputs string JSON values - add parsing for it ([c190594](https://github.com/tschinz/lq/commit/c190594bb79eb507e3ea96f23a96609d1e3a8dcd) - Axam)
- *(hint)* Fix hint rendering ([414ba2e](https://github.com/tschinz/lq/commit/414ba2e785df35a240cf4b1606bf3d129726fdf9) - Borgeat Rémy)
- *(clippy)* Fix linter issues ([9bd6fcc](https://github.com/tschinz/lq/commit/9bd6fcc4ee078adc846beb119e2c439a571e0b2f) - Borgeat Rémy)
- Starting message ([eb1c175](https://github.com/tschinz/lq/commit/eb1c175d1ff3dc5db635c8cbc24e933739811ff4) - zas)
- Overview current exercise highlight ([8672eb9](https://github.com/tschinz/lq/commit/8672eb96363cda2330ef58866c8c6a1d5a8e78b8) - zas)
- *(ci)* Fix clippy and fmt issues ([624cbbf](https://github.com/tschinz/lq/commit/624cbbf7c2e730b544e111f4f8187e1bf64519e3) - Borgeat Rémy)
- *(just)* Possible arguments for run-test recipe ([1dcc4e4](https://github.com/tschinz/lq/commit/1dcc4e4cfc5fbd129d5d4cc96f96d74ecc7641ba) - zas)
- *(rust)* Clippy / fmt ([4aeb2fa](https://github.com/tschinz/lq/commit/4aeb2fa7b8cdcb36620700bdc4e2d03a02d1739a) - Axam)
- *(build)* French strings ([608b595](https://github.com/tschinz/lq/commit/608b595965db2be059ef051f21e4fb0d7a44f7c5) - axel.amand)
- Readme code block ([3b75e0c](https://github.com/tschinz/lq/commit/3b75e0c3ad3695dcbb0f3d5d5a33c370985062b3) - zas)
- *(overview)* Don't reset overview cursor position ([c769943](https://github.com/tschinz/lq/commit/c76994320d8dc1f62cd7c0b0aebfe26a4794fbd4) - mel-64)
- *(status)* Bail out if no exercise can be found in repo directory ([8e28f57](https://github.com/tschinz/lq/commit/8e28f57d9ac08e112025a1200506af095149509e) - mel-64)
- Fix excercise watcher not working reliably with certain editors on linux ([c92647f](https://github.com/tschinz/lq/commit/c92647f15d187338e628097c6982fc63f879a958) - mel-64)

### 💼 Other

- Initial commit ([fd1e700](https://github.com/tschinz/lq/commit/fd1e7003d4638a9f968e302a5b1d689b1ac01f46) - tschinz)
- Riscv example to expect ABI names ([37887ba](https://github.com/tschinz/lq/commit/37887ba6571054af42ca91c05df9c356625b7aae) - Axam)
- Merge pull request #2 from tschinz/hotfix/ripes_parsing

Fix: Ripes parsing for ABI names ([1c77321](https://github.com/tschinz/lq/commit/1c77321c01cb44594a804ce594a5147c2a9416db) - BorgeatRemy)
- Revert "Fix: Ripes parsing for ABI names" ([24ea6c2](https://github.com/tschinz/lq/commit/24ea6c233347be5fcfb0cf55b2f52ebfee9e586d) - tschinz)
- Merge branch 'main' of https://github.com/tschinz/langquest ([a298c0f](https://github.com/tschinz/lq/commit/a298c0fb343715cbe699cbbc924931bd0dfaadb3) - Borgeat Rémy)
- Riscv example to expect ABI names ([bb1213a](https://github.com/tschinz/lq/commit/bb1213aef4e77ac21646ded693be9ac5573fdd46) - Axam)
- Merge pull request #4 from tschinz/feat/progressBarV2

feat(progressbar): Refactor progress bar to display exercices status ([343c462](https://github.com/tschinz/lq/commit/343c462da1144a9f7206e4fb2f9979f3279e3da1) - BorgeatRemy)
- Merge branch 'main' of https://github.com/tschinz/langquest ([ed4bcec](https://github.com/tschinz/lq/commit/ed4bcecdfab114577ba263dcb6005fdd67ab95f7) - Borgeat Rémy)
- Merge remote-tracking branch 'origin/test/deferred_verify' ([36e7947](https://github.com/tschinz/lq/commit/36e79471ead6b16d7f03adc19c9b8b50feb63b53) - zas)

### 🚜 Refactoring

- *(md)* Remove custon syntax highlighting and its config ([fe5b7c1](https://github.com/tschinz/lq/commit/fe5b7c181e0d90db9869111af7fcd3b40e912737) - zas)
- *(status)* -s —stats and status is now the same ([246b060](https://github.com/tschinz/lq/commit/246b060f78e3bf3959c13dc474738ffc685a9218) - zas)
- *(ci)* Default vars taken from .env.example instead ([2630d1b](https://github.com/tschinz/lq/commit/2630d1b5e8880d60fe7a533bad8ce6b2db638198) - axel.amand)

### 🧪 Testing

- *(sample)* Added sample exercises for rust, go, python, riscv and markdown ([0f80c92](https://github.com/tschinz/lq/commit/0f80c92123e691a4b14ce9b5fe6381e6b06dff7a) - zas)
- *(sample-repo)* Simplify tests ([ab5a9e4](https://github.com/tschinz/lq/commit/ab5a9e4aa252823adbc028cfe64fabb16945b869) - zas)
- *(solutions)* Remove solutiuons from tests ([099a1fe](https://github.com/tschinz/lq/commit/099a1fe5dbb4dd9ddad5cf24c33904b5e53e2bc1) - zas)
- *(integration)* Fix integration test ([5861366](https://github.com/tschinz/lq/commit/58613669d17ebbabba07729db5c3d7eab7d1331f) - zas)
- *(sample-repo)* Fix rust example ([5c92c9f](https://github.com/tschinz/lq/commit/5c92c9fbd04017707d22b2857d2668a20bbdfe7d) - zas)
- *(markdown)* Add markdown example ([b1b7fd9](https://github.com/tschinz/lq/commit/b1b7fd9cda3f4a09ac23a764d59fd43d10d47454) - zas)

### ⚙️ Miscellaneous Tasks

- Add readme, license file, cliff config, cargo config, just file, gitignore ([45dcd9b](https://github.com/tschinz/lq/commit/45dcd9bacd20884d3e03d5b088f956f5686a3d80) - zas)
- *(ci)* Add github actions ([e4c6c7b](https://github.com/tschinz/lq/commit/e4c6c7bcf4b939ff75e1810317a0a0760a59a3fe) - zas)
- *(readme)* Adapted readme to the latest version ([5278dc4](https://github.com/tschinz/lq/commit/5278dc40ef71fb395b98cf91a2a885faac9e573c) - zas)
- *(fmt)* Add rustfmt config ([96e5772](https://github.com/tschinz/lq/commit/96e5772148145c6243523821829ca54629b5a6bd) - zas)
- *(publish)* Add cargo publish information ([76af231](https://github.com/tschinz/lq/commit/76af231e979e517473dfe183fd4b2a673478fc6d) - zas)
- *(fmt)* Apply rustfmt settings ([c0d6212](https://github.com/tschinz/lq/commit/c0d6212444d38c4898608a4dceeafea964a65424) - zas)
- *(demo)* Add demo gif ([b0043a7](https://github.com/tschinz/lq/commit/b0043a79578036f2e6b7cf0286949af042ee89e8) - zas)
- *(readme)* Minor fixes ([8407b10](https://github.com/tschinz/lq/commit/8407b10e81f968e352ebd278417adbf97e7c8135) - zas)
- *(python)* Remove unnecesary lines ([5b57255](https://github.com/tschinz/lq/commit/5b5725528f3fc56ec0cc428f172aa40bb02b6e66) - zas)
- Clippy and fmt ([f217c04](https://github.com/tschinz/lq/commit/f217c04dd918b4614ca563ebb2151c5a3ce02e90) - zas)
- *(fmt)* Fix rustfmt ([681c803](https://github.com/tschinz/lq/commit/681c803a08f927b06f9f459b12d78324330e88c9) - zas)
- *(python)* Fix ruff message ([3aad522](https://github.com/tschinz/lq/commit/3aad522a0000e948b674ede70f1d7fd1febc29fc) - zas)
- *(ci)* Remove safeguards and let test fail ([fccdde0](https://github.com/tschinz/lq/commit/fccdde09f97e65c5b6e13747caaecc9554598432) - zas)
- *(readme)* Add cpp in readme ([09402d4](https://github.com/tschinz/lq/commit/09402d406561bb2112fe7ea3bc470c3034e0692a) - zas)
- *(dependencies)* Bump dependencies ([79ef075](https://github.com/tschinz/lq/commit/79ef075c5a6ccd77a588cb5e0afa421ee530733b) - zas)
- *(ignore)* Ignore sample-test progress file ([64ad8c0](https://github.com/tschinz/lq/commit/64ad8c0575f3f6e723412296db0c67f93fbee400) - zas)
- *(release)* Bump version to 1.0.0 and update packages ([bcf3336](https://github.com/tschinz/lq/commit/bcf33368d4daa1e0d50c62d72023b5d2e8b29905) - zas)
- *(release)* Fix cargo and doc warnings ([883b23f](https://github.com/tschinz/lq/commit/883b23f374eab96cce804fbd5bfd293d5cffa8fa) - zas)



