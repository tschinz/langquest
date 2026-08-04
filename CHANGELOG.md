# Changelog

All notable changes to this project will be documented in this file.

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



