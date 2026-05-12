# Changelog

## [0.18.3](https://github.com/pmqueiroz/nova/compare/v0.18.2...v0.18.3) (2026-05-12)


### Bug Fixes

* chain PROMPT_COMMAND instead of overwriting, drop --norc for zoxide compat ([5da0801](https://github.com/pmqueiroz/nova/commit/5da080139bad095b50e41c57367453d5dcedda52))
* **ui:** skip newlines for wrapped lines in selection copy ([a09ebed](https://github.com/pmqueiroz/nova/commit/a09ebed819fdf0740bd6128782c038eb12421339))

## [0.18.2](https://github.com/pmqueiroz/nova/compare/v0.18.1...v0.18.2) (2026-05-12)


### Bug Fixes

* **core:** reflow wrapped lines on terminal resize ([2888c98](https://github.com/pmqueiroz/nova/commit/2888c985a00876d04d818c065aa292fb5d91a6fd))

## [0.18.1](https://github.com/pmqueiroz/nova/compare/v0.18.0...v0.18.1) (2026-05-12)


### Bug Fixes

* remove open ssh conversion on aur publishing ([b2ebf0a](https://github.com/pmqueiroz/nova/commit/b2ebf0ae4c73348698b35f84cb0c53d45577328e))

## [0.18.0](https://github.com/pmqueiroz/nova/compare/v0.17.2...v0.18.0) (2026-05-12)


### Features

* **cli:** implement 'nova explain' command ([a8375ef](https://github.com/pmqueiroz/nova/commit/a8375ef4f82233520717eb01c3f87d5b3dce06fc))

## [0.17.2](https://github.com/pmqueiroz/nova/compare/v0.17.1...v0.17.2) (2026-05-12)


### Bug Fixes

* **ui:** keep rounded corners on macos when maximized ([765821c](https://github.com/pmqueiroz/nova/commit/765821c13b9b09fdef1eb5d25210d4d6b93b553b))

## [0.17.1](https://github.com/pmqueiroz/nova/compare/v0.17.0...v0.17.1) (2026-05-12)


### Bug Fixes

* **core:** use $SHELL as the primary default shell fallback on Unix ([f556a3d](https://github.com/pmqueiroz/nova/commit/f556a3d39c32157e76a7d7fff815a3b8a13dc4c1))
* sync window maximized state with OS and fix border artifacts ([82035e4](https://github.com/pmqueiroz/nova/commit/82035e4297a19d93db319a7ab8bcdd2782d79c03))

## [0.17.0](https://github.com/pmqueiroz/nova/compare/v0.16.0...v0.17.0) (2026-05-11)


### Features

* **cli:** auto-submit ask_ai when preset content is provided ([1b8a538](https://github.com/pmqueiroz/nova/commit/1b8a538f18a2f933c3db066556116b319e645458))


### Bug Fixes

* run bash startup with interactive flag ([abd31d6](https://github.com/pmqueiroz/nova/commit/abd31d648b9e17a558d939768530596011f14828))

## [0.16.0](https://github.com/pmqueiroz/nova/compare/v0.15.2...v0.16.0) (2026-05-11)


### Features

* start fish interactive ([06e0067](https://github.com/pmqueiroz/nova/commit/06e006725b9e13689e9512ef8eeb006450c33c65))
* start unix common shells as login and interactive ([bb2f362](https://github.com/pmqueiroz/nova/commit/bb2f362c033e4f4fe91bd178db4d4afad63d47f0))


### Bug Fixes

* correct indentation of CASK EOF in release workflow ([b205ae7](https://github.com/pmqueiroz/nova/commit/b205ae76319a2557bcd2d5273d9a8f46d4b47962))
* ensure PS1 work correctly ([b195488](https://github.com/pmqueiroz/nova/commit/b195488fb472734910f2381f6a09915cddda81b0))
* prevent bash to lose PS1 on login ([223dc76](https://github.com/pmqueiroz/nova/commit/223dc76bfabf15b835d55f0fcca84bb486554d5e))

## [0.15.2](https://github.com/pmqueiroz/nova/compare/v0.15.1...v0.15.2) (2026-05-11)


### Bug Fixes

* nsis uninstall path removal ([5b7841e](https://github.com/pmqueiroz/nova/commit/5b7841efb45d7aa6d20ac10435ce7916b18441f4))

## [0.15.1](https://github.com/pmqueiroz/nova/compare/v0.15.0...v0.15.1) (2026-05-11)


### Bug Fixes

* installer mode typo ([0d6f193](https://github.com/pmqueiroz/nova/commit/0d6f1937b1fcb324370172c2f80e270daaf4dfa3))

## [0.15.0](https://github.com/pmqueiroz/nova/compare/v0.14.0...v0.15.0) (2026-05-11)


### Features

* add nova to bin on install ([2c28be3](https://github.com/pmqueiroz/nova/commit/2c28be31b7524e295f911b72aba8ff70ff744fad))
* open ask ai with cli command ([b32c549](https://github.com/pmqueiroz/nova/commit/b32c5497d4c139578ba09329f300e89ba0be07cd))

## [0.14.0](https://github.com/pmqueiroz/nova/compare/v0.13.0...v0.14.0) (2026-05-10)


### Features

* implement bracketed paste mode (CSI ?2004h/l) ([746941a](https://github.com/pmqueiroz/nova/commit/746941a3b249efd34fa791f9a271bf0f1e5cb7b2)), closes [#27](https://github.com/pmqueiroz/nova/issues/27)
* implement mouse reporting (CSI ?1000h / ?1006h SGR mode) ([3177935](https://github.com/pmqueiroz/nova/commit/31779357f867a7059b2baa9d329d141149b1662f)), closes [#28](https://github.com/pmqueiroz/nova/issues/28)
* implement text attributes (bold, italic, underline, dim, blink, strikethrough) ([c86b1f4](https://github.com/pmqueiroz/nova/commit/c86b1f4bd2e20917ad18388da84c556a40121a2e)), closes [#26](https://github.com/pmqueiroz/nova/issues/26)


### Bug Fixes

* convert AUR SSH key to OPENSSH format to fix libcrypto PEM decoding error in Arch container ([10f41b9](https://github.com/pmqueiroz/nova/commit/10f41b9941de7abf25aa5e0ac8ccac8f7f277b55))
* verify and harden scroll region enforcement (CSI r) ([44e2a22](https://github.com/pmqueiroz/nova/commit/44e2a228aab019f1b6281d6e163c512e8f0df51e)), closes [#30](https://github.com/pmqueiroz/nova/issues/30)


### Performance Improvements

* flatten grid allocator from Vec&lt;Vec&lt;Cell&gt;&gt; to Vec&lt;Cell&gt; with stride indexing ([af41848](https://github.com/pmqueiroz/nova/commit/af41848c591005970e494758418a949872d32ad0)), closes [#29](https://github.com/pmqueiroz/nova/issues/29)

## [0.13.0](https://github.com/pmqueiroz/nova/compare/v0.12.5...v0.13.0) (2026-05-09)


### Features

* implement OSC 8 hyperlinks ([b302dd6](https://github.com/pmqueiroz/nova/commit/b302dd64c3c7617b17fdcb60fc8f2d8a8372548e)), closes [#31](https://github.com/pmqueiroz/nova/issues/31)

## [0.12.5](https://github.com/pmqueiroz/nova/compare/v0.12.4...v0.12.5) (2026-05-09)


### Bug Fixes

* force overwrite Packages.gz in apt repo job ([74b24ed](https://github.com/pmqueiroz/nova/commit/74b24ed182da757f1f9c4e9923ffe05c8fffe4f0))

## [0.12.4](https://github.com/pmqueiroz/nova/compare/v0.12.3...v0.12.4) (2026-05-09)


### Bug Fixes

* correct AUR action, RPM git add path, winget continue-on-error ([24c822f](https://github.com/pmqueiroz/nova/commit/24c822f8bf00fdfb3c4fd460ae784cdf07a44da7))

## [0.12.3](https://github.com/pmqueiroz/nova/compare/v0.12.2...v0.12.3) (2026-05-09)


### Bug Fixes

* add missing package metadata fields for cargo-generate-rpm ([12a2fcd](https://github.com/pmqueiroz/nova/commit/12a2fcd34fd71b0adf2eaa3a925c4630475f7fd9))

## [0.12.2](https://github.com/pmqueiroz/nova/compare/v0.12.1...v0.12.2) (2026-05-09)


### Bug Fixes

* add missing license field for cargo-generate-rpm ([cb89607](https://github.com/pmqueiroz/nova/commit/cb8960708c4173421d45280f9728ff736671468a))

## [0.12.1](https://github.com/pmqueiroz/nova/compare/v0.12.0...v0.12.1) (2026-05-09)


### Bug Fixes

* rpm pckg ([ce0166b](https://github.com/pmqueiroz/nova/commit/ce0166b71e5ff7a6b8bb29a636eb5604a6796504))

## [0.12.0](https://github.com/pmqueiroz/nova/compare/v0.11.0...v0.12.0) (2026-05-09)


### Features

* detect ssh connection ([4961e60](https://github.com/pmqueiroz/nova/commit/4961e60a23656a70cd35e5e9582ef92f2cdfa20d))
* word and line backward and forward ([2e1db38](https://github.com/pmqueiroz/nova/commit/2e1db38cf8314d76193397df4c4435ebb01906a6))

## [0.11.0](https://github.com/pmqueiroz/nova/compare/v0.10.1...v0.11.0) (2026-05-09)


### Features

* bell ([5bc119c](https://github.com/pmqueiroz/nova/commit/5bc119ccabae34c1a5bc7066706ee585030e9a82))
* exit on esc on modals ([f0a958c](https://github.com/pmqueiroz/nova/commit/f0a958c2120dc9dc543966da477ab712240a60e1))
* standardizes the icons ([7191c54](https://github.com/pmqueiroz/nova/commit/7191c54b70265c0930a6eae487bf9481febd4a9f))


### Bug Fixes

* data entry alignments ([bc91e7a](https://github.com/pmqueiroz/nova/commit/bc91e7aa34f775141f24869381adb6001b0af184))

## [0.10.1](https://github.com/pmqueiroz/nova/compare/v0.10.0...v0.10.1) (2026-05-09)


### Bug Fixes

* missing icon on linux ([9ba8347](https://github.com/pmqueiroz/nova/commit/9ba83477cdf96efac88d3bbdd8e65f6571d87459))
* start hidden on windows to prevent flicker ([9bc8b29](https://github.com/pmqueiroz/nova/commit/9bc8b2998f07c78f4ee90bdcf4029e5b1f0c3552))

## [0.10.0](https://github.com/pmqueiroz/nova/compare/v0.9.0...v0.10.0) (2026-05-09)


### Features

* add ai agentic features ([87245d0](https://github.com/pmqueiroz/nova/commit/87245d02d125cabeca2807aa2e9a413e00214912))
* resize cursor ([ce792f9](https://github.com/pmqueiroz/nova/commit/ce792f918fb18d064e30bc5c7a5ad07c5a7941ac))
* select window control style ([45d4ed4](https://github.com/pmqueiroz/nova/commit/45d4ed4098d7a0f403cfdd84a2699c47f4a1d01b))


### Bug Fixes

* prevent relaunch shell on minimize on Windows ([6a03406](https://github.com/pmqueiroz/nova/commit/6a03406fb62c58a11375d616e4fb1ac6069e84fd))
* prevent relaunch shell on resize ([b94a559](https://github.com/pmqueiroz/nova/commit/b94a5591f9ce39751832c7d040a7cb3ae47c9910))
* roundend corners on linux ([754fcea](https://github.com/pmqueiroz/nova/commit/754fcea832616678eac48a0f04fbaf8f0b4294d9))

## [0.9.0](https://github.com/pmqueiroz/nova/compare/nova-v0.8.0...nova-v0.9.0) (2026-05-08)


### Features

* detect and open url ([cee3925](https://github.com/pmqueiroz/nova/commit/cee39254e8e3dbcc52f0c7f05cfad96444cc904f))


## [0.8.0](https://github.com/pmqueiroz/nova/compare/v0.7.0...v0.8.0) (2026-05-08)


### Features

* colored prompt ([26c067c](https://github.com/pmqueiroz/nova/commit/26c067c9fc5f64a04ce929502c3416adbe3de14d))
* improve title bar visual ([514aa15](https://github.com/pmqueiroz/nova/commit/514aa152b5f99c86ec2ae0f934ad312c35292d04))
* scrollback buffer ([df68acf](https://github.com/pmqueiroz/nova/commit/df68acf85bd2dc8085c9bb6e97b1f61a293f168d))


### Bug Fixes

* macos icon too big ([a632509](https://github.com/pmqueiroz/nova/commit/a632509816f7f388effe95d30e0571817b16ccc6))
* remove selection offset from cursor ([59015e4](https://github.com/pmqueiroz/nova/commit/59015e4d9d56690c5e59e409a8de1ad7e6426f43))

## [0.7.0](https://github.com/pmqueiroz/nova/compare/v0.6.0...v0.7.0) (2026-05-08)


### Features

* add 24 bit true color support ([43030b0](https://github.com/pmqueiroz/nova/commit/43030b031e7f3c8daba15b20433f04be6bb05b20))
* add wsl and git bash support ([21e1d69](https://github.com/pmqueiroz/nova/commit/21e1d69c68acef9e4e5acd24698e7d3cf8920904))
* ansi colors palette ([8397d35](https://github.com/pmqueiroz/nova/commit/8397d35a8e52ca1c589bfebbcbd9b0f8afdc0521))
* fit grid better in term area ([c3839ce](https://github.com/pmqueiroz/nova/commit/c3839ceb7b3ed0ea2d4c66699cf6e3b87ba9df47))

## [0.6.0](https://github.com/pmqueiroz/nova/compare/v0.5.4...v0.6.0) (2026-05-08)


### Features

* configure date and time format ([2395192](https://github.com/pmqueiroz/nova/commit/239519273b8f0f595cec397b7f320094ed4abb8b))
* settings pannel ([f4e8253](https://github.com/pmqueiroz/nova/commit/f4e82538dd1c113bc4dcbba3fc3fdf11e5f7c554))
* shell picker ([20328cf](https://github.com/pmqueiroz/nova/commit/20328cfcaf36437fc0f67d56be986642be243d57))


### Bug Fixes

* default font name ([21b9c46](https://github.com/pmqueiroz/nova/commit/21b9c46edad22630a9fc0f8a48eb2fe7266de870))

## [0.5.4](https://github.com/pmqueiroz/nova/compare/v0.5.3...v0.5.4) (2026-05-07)


### Bug Fixes

* macos 1024 icon syntax ([a136461](https://github.com/pmqueiroz/nova/commit/a136461673f173c37e2deb66a5512b33d4adf00b))

## [0.5.3](https://github.com/pmqueiroz/nova/compare/v0.5.2...v0.5.3) (2026-05-07)


### Bug Fixes

* remove 64 mc icon ([fdb6bd6](https://github.com/pmqueiroz/nova/commit/fdb6bd603f91fd050a9f7d6dcbd97d9f861281bd))

## [0.5.2](https://github.com/pmqueiroz/nova/compare/v0.5.1...v0.5.2) (2026-05-07)


### Bug Fixes

* build packages ([172f356](https://github.com/pmqueiroz/nova/commit/172f356b758653dbd4e4bb820ee9278ea6e4ba85))

## [0.5.1](https://github.com/pmqueiroz/nova/compare/v0.5.0...v0.5.1) (2026-05-07)


### Bug Fixes

* windows packager ([0542ab9](https://github.com/pmqueiroz/nova/commit/0542ab994dca634edc6910c50ee6a92437b38241))

## [0.5.0](https://github.com/pmqueiroz/nova/compare/v0.4.0...v0.5.0) (2026-05-07)


### Features

* char bg ([2d761e7](https://github.com/pmqueiroz/nova/commit/2d761e7a54c54e163c5fcfc2ace4b7a3a9de809c))
* close tab on middle press ([6ec9020](https://github.com/pmqueiroz/nova/commit/6ec90204e8fef948532eb40e8138b412cf1d71a3))
* config file ([51f8bbd](https://github.com/pmqueiroz/nova/commit/51f8bbd7a714c07f1940524b07163165d7f5f077))
* implement full VT emulation for TUI app support ([eefdadd](https://github.com/pmqueiroz/nova/commit/eefdadd40871c029854d7e8454115cc5eeaa9d20))
* macos default settings ([f6b5c46](https://github.com/pmqueiroz/nova/commit/f6b5c460b424b5567c2b27f49ffceebfa8314a48))
* open config file ([5097ed0](https://github.com/pmqueiroz/nova/commit/5097ed0d0d6dbc37e5d9435462687c34873c22ce))
* select and copy ([c4130f2](https://github.com/pmqueiroz/nova/commit/c4130f22679c329e35a9732503934e06775d86a1))


### Bug Fixes

* quote file on openning ([71fefae](https://github.com/pmqueiroz/nova/commit/71fefaee7fa9b31e641328dbe1883e804a6571ce))
* traffic lights padding ([fb0b444](https://github.com/pmqueiroz/nova/commit/fb0b44466218f8c14ad26f28628770077586fbcd))

## [0.4.0](https://github.com/pmqueiroz/nova/compare/v0.3.1...v0.4.0) (2026-05-07)


### Features

* tab management shortcuts ([326a9d5](https://github.com/pmqueiroz/nova/commit/326a9d5a947b20164e99f34ec08efe32b5f0d317))

## [0.3.1](https://github.com/pmqueiroz/nova/compare/v0.3.0...v0.3.1) (2026-05-07)


### Bug Fixes

* close tab button ([82fbcf5](https://github.com/pmqueiroz/nova/commit/82fbcf563a8b250b44b6d31edcbd29373aad2539))
* prevent redraw on first paint ([109473e](https://github.com/pmqueiroz/nova/commit/109473e29d90435f7519e436b25c39490f933880))

## [0.3.0](https://github.com/pmqueiroz/nova/compare/v0.2.0...v0.3.0) (2026-05-07)


### Features

* better coloring ([c591b2c](https://github.com/pmqueiroz/nova/commit/c591b2c4eaa9869c1c367efd1c9779c836aed6b3))
* better cursor ([374e9e4](https://github.com/pmqueiroz/nova/commit/374e9e4ab3338164683f35bed2aed9a5dbb93a74))
* write term content with rich text ([7bc874e](https://github.com/pmqueiroz/nova/commit/7bc874e831c54ecf870122c2f107237def4a540c))


### Bug Fixes

* advance on tab ([201808f](https://github.com/pmqueiroz/nova/commit/201808fbe59db0b18ed8d31fa3895f992b015557))
* rounded window on macos ([a2c179f](https://github.com/pmqueiroz/nova/commit/a2c179ff9e07bc2136a56157a7a88e5afd31b4c6))

## [0.2.0](https://github.com/pmqueiroz/nova/compare/v0.1.0...v0.2.0) (2026-05-07)


### Features

* add controls ([396123d](https://github.com/pmqueiroz/nova/commit/396123d81219143c900c89c748a220c129395f5b))
* add mark to title bar ([d7f8b75](https://github.com/pmqueiroz/nova/commit/d7f8b7599cc23a806729f8097e8a0cef6bb0162a))
* relayout title and tab bar ([ef5e26f](https://github.com/pmqueiroz/nova/commit/ef5e26f61063ec983d2871a0e8e680c1b78a6b2f))
* remove shell icon ([70f505b](https://github.com/pmqueiroz/nova/commit/70f505b79b0ed76cabec0a4f4a2305eef831af5c))
* rounded corner on windows ([60bdbb5](https://github.com/pmqueiroz/nova/commit/60bdbb55bb4d7b2eaa5b09bc5b7c6522d149d298))


### Bug Fixes

* hide cmd on prod build ([41cdb97](https://github.com/pmqueiroz/nova/commit/41cdb97d3a2e5fd3428213550ba1d7cd59e7cc95))
* shell and pwd status on windows ([3d9b648](https://github.com/pmqueiroz/nova/commit/3d9b6488512af7209e4cb34f53e96ee8815499f3))
* window resize on windows ([298a7c0](https://github.com/pmqueiroz/nova/commit/298a7c0f522779a005964819e1ce3b460671186e))
* wrong icon on macos ([c37d243](https://github.com/pmqueiroz/nova/commit/c37d243c99222971643a07588390d619a33ea7ba))

## 0.1.0 (2026-05-07)


### Features

* add properly support for powershell ([2452a91](https://github.com/pmqueiroz/nova/commit/2452a912366873e86cd1380099b32333be729f93))
* adjust grid on resize ([7cb72d3](https://github.com/pmqueiroz/nova/commit/7cb72d3147b6cb5d0ad6959d999fcfdc55c0c2ec))
* agent status ([47aa9ab](https://github.com/pmqueiroz/nova/commit/47aa9ab0ede3863751a79f6a13a2d23f5183841c))
* app icons ([e9c9af3](https://github.com/pmqueiroz/nova/commit/e9c9af34c95cef5020cbe10e42536c5f8eb6a38c))
* better tab icons ([adb9e2a](https://github.com/pmqueiroz/nova/commit/adb9e2aa40d48a2b36012263a32892d6e1a74b5a))
* connect to pty ([bf66e2c](https://github.com/pmqueiroz/nova/commit/bf66e2c1d400597873996103b9f690d0bc2c7bc7))
* create basic window ([791cd44](https://github.com/pmqueiroz/nova/commit/791cd443f91e0da8fa71b7912f91dd9da6e924db))
* display statuses ([53bdb1a](https://github.com/pmqueiroz/nova/commit/53bdb1ac98de24b27b161216bf63fc637c15f4cc))
* fixed tab width ([b64fa0f](https://github.com/pmqueiroz/nova/commit/b64fa0f2e39d361855e26a6e58cc0d8c46db23ec))
* grid and ansi parser ([e678b0a](https://github.com/pmqueiroz/nova/commit/e678b0a149b54c7e7507326832cda6e0c03f109a))
* icons ([231c83c](https://github.com/pmqueiroz/nova/commit/231c83c5969a8b5382dc4e7d00d63283272def1d))
* implement missing csi dispatches ([7f223ed](https://github.com/pmqueiroz/nova/commit/7f223eddcd0de3dda371448015c80fe54a49a01f))
* improve status bar style ([efe7cef](https://github.com/pmqueiroz/nova/commit/efe7cefe42ffa45369c7515caa6c969d988137bc))
* powershell prompt ([17bbafc](https://github.com/pmqueiroz/nova/commit/17bbafc03e9545e5d9959f378e58b61a34c1bd17))
* raw grid instead of input ([9139357](https://github.com/pmqueiroz/nova/commit/9139357d0341ee088cef033614010f5f87fdac0e))
* scrollable term ([2d3ac31](https://github.com/pmqueiroz/nova/commit/2d3ac31d8de71667757eb45be92d08d297e57c52))
* setup font ([d60b24f](https://github.com/pmqueiroz/nova/commit/d60b24ffeec95bf1a9391549e39d39034d8ac89a))
* status bar ([19da9b4](https://github.com/pmqueiroz/nova/commit/19da9b4102dcfb6f6ba7bd61012d7a1e13c0555c))
* tab bar comp ([80654aa](https://github.com/pmqueiroz/nova/commit/80654aabd0d38ca53038823105ba8d7d47162806))
* tabs ([4bd2950](https://github.com/pmqueiroz/nova/commit/4bd295038c63f63a4de3f173e53ce1938a9dae19))
* term component ([d898b4c](https://github.com/pmqueiroz/nova/commit/d898b4ccd088ca23995a6f8af69bd4f94324f5f4))
* title bar comp ([6ce6789](https://github.com/pmqueiroz/nova/commit/6ce6789264108c856e26df81bf3454799c8bb90f))
* use only ps1 instead of made up prompt ([eeec211](https://github.com/pmqueiroz/nova/commit/eeec2119c9caa9b282b364395400e1d8cdb26256))


### Bug Fixes

* backspace actually remove chars ([be236e2](https://github.com/pmqueiroz/nova/commit/be236e2cd0e38b49f714c6b16b5ed149d88d9704))
* broken double quote ([844864e](https://github.com/pmqueiroz/nova/commit/844864e3f51c366237ecb5f430bf7fed597bff78))


### Performance Improvements

* use str buffer to render grid ([20dc77b](https://github.com/pmqueiroz/nova/commit/20dc77b7f63a25f39a1d6155b6909ed6f988a8ea))
