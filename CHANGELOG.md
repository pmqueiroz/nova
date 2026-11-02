# Changelog

## [1.2.0](https://github.com/pmqueiroz/nova/compare/v1.1.1...v1.2.0) (2026-11-02)


### Features

* **input:** show running state in the input bar with a sliding prompt ([66ae2ef](https://github.com/pmqueiroz/nova/commit/66ae2ef853359787c7b17b6701b54629bfe3f500))
* **tabs:** drag a tab to the window edge to open it as a split pane ([8e22806](https://github.com/pmqueiroz/nova/commit/8e22806d772fb57c71957c43b4098fb2f34e3709))
* **tabs:** label tabs with the directory name instead of the full path ([c170377](https://github.com/pmqueiroz/nova/commit/c17037784b42963e56001ab1e40f9a78898c688e))
* **tabs:** show a skeleton thumbnail of the tab while dragging it ([0c79298](https://github.com/pmqueiroz/nova/commit/0c7929879e600537f632ef541f0dda79603a5dcd))


### Bug Fixes

* **blocks:** select text and forward clicks in full-screen programs ([adcf89a](https://github.com/pmqueiroz/nova/commit/adcf89a2e9920c0e466f21434ee765114bd36194))
* **input:** put the cursor at the end when recalling history ([9f2ef31](https://github.com/pmqueiroz/nova/commit/9f2ef31c38122b7ac984bf063caf708664c42eaf))
* **tabs:** keep the shell alive when a pane moves between tab and split ([6b92766](https://github.com/pmqueiroz/nova/commit/6b927665945ca3b1d4e14278019e3eee0549c6bf))
* **ui:** drop the iced suffix from the window title ([c8edeb7](https://github.com/pmqueiroz/nova/commit/c8edeb7a0debc1262a41f8c2067fafd18c6b6885))
* **windows:** keep the first character of the first powershell command ([0103a47](https://github.com/pmqueiroz/nova/commit/0103a47a31353f6fc00c409184ed1e16a3c73294))
* **windows:** silence the powershell error ding unless the bell is audio ([f2f60f1](https://github.com/pmqueiroz/nova/commit/f2f60f1f5627cc4edb3c714438a62ff464d8b5bc))


### Performance Improvements

* **bell:** play one bell at a time instead of a thread per bel ([72136f0](https://github.com/pmqueiroz/nova/commit/72136f08cf54b1818d4b61ae0da5b57f5eb5ab2a))
* **block:** freeze the visible line range of finished blocks ([bc1fc9e](https://github.com/pmqueiroz/nova/commit/bc1fc9e93d7b6d5b7c8a787baf1998ebbc9aa534))
* **block:** report pointer moves over block cards only when the hovered cell changes ([328a3bb](https://github.com/pmqueiroz/nova/commit/328a3bbbe73beea89d0bb651c6b9f469a0cee182))
* **block:** resolve block lines once when collecting text and copying selections ([9370aa1](https://github.com/pmqueiroz/nova/commit/9370aa1444347858a0218816b9599006f5a1d790))
* **build:** enable fat lto and single codegen unit for release builds ([48392c0](https://github.com/pmqueiroz/nova/commit/48392c00a024f3240b9a8a463e7d6e9be350f5f3))
* **completion:** cache path executables per directory until it changes ([7ae8eee](https://github.com/pmqueiroz/nova/commit/7ae8eee7a930eadf455ef805d0cef2fff288a97d))
* **completion:** use the directory entry's file type instead of a stat per entry ([f083612](https://github.com/pmqueiroz/nova/commit/f083612b114cb55bd521ac4559c1c414f51da2dc))
* **glyph:** look up block element rects without allocating ([165d396](https://github.com/pmqueiroz/nova/commit/165d39671582a657a9dbd41ac78bc4293cce98d5))
* **grid:** erase with slice fills and reuse evicted scrollback rows ([8bd389a](https://github.com/pmqueiroz/nova/commit/8bd389a54e127728842b78d95e4dac155bd0467b))
* **grid:** find history suggestions in one scan instead of building a map per keystroke ([0d67981](https://github.com/pmqueiroz/nova/commit/0d67981b0fb24d32851e57fc0f428fb96926a6b1))
* **grid:** move scrollback rows into reflow instead of cloning them ([00f4afd](https://github.com/pmqueiroz/nova/commit/00f4afd17014aece29f246fd30e2bff7f01cbd85))
* **grid:** pack cell colors into three bytes to shrink cells from 88 to 56 bytes ([71d72f9](https://github.com/pmqueiroz/nova/commit/71d72f95f0243ab8575a8d009981c36d319cafd2))
* **grid:** scroll the screen by rotating a row ring instead of moving every cell ([67fbc8d](https://github.com/pmqueiroz/nova/commit/67fbc8d6b8e40f1b8294f1e4b80cb343ee699a01))
* **grid:** store cell text inline instead of heap-allocating per cell ([c6c2121](https://github.com/pmqueiroz/nova/commit/c6c21213e159bf93cee80e6d8db15f71bb6093ea))
* **paste:** preview a paste without collecting every line each frame ([d0d0df1](https://github.com/pmqueiroz/nova/commit/d0d0df130a4e0f0d7f0e98771e1f58b594482652))
* **pty:** cache the debug-log env flag and skip unchanged cwd clones ([efd7fd3](https://github.com/pmqueiroz/nova/commit/efd7fd3118dd585a67173b3bf404877f13cd7071))
* **pty:** read 64 KiB at a time and batch queued output into one message ([f906d05](https://github.com/pmqueiroz/nova/commit/f906d0594b2650538fdff6ba4393274a42e0c306))
* **router:** feed vte in segments instead of one byte at a time ([9ecf23d](https://github.com/pmqueiroz/nova/commit/9ecf23d2a84867582000f2ffda65cbee41ac199f))
* **startup:** detect installed shells in the background instead of before the window opens ([09f5858](https://github.com/pmqueiroz/nova/commit/09f58588df92db2ddd089ff8e06c271124b23bd7))
* **tabs:** stop reading the git branch from disk on every pty output ([e5fe963](https://github.com/pmqueiroz/nova/commit/e5fe963df0c2aa1d6abe5f89694fb444479dcda4))
* **term:** draw the grid from one widget instead of a widget per cell ([30c1d04](https://github.com/pmqueiroz/nova/commit/30c1d0491ee6ea0d2610bc8a0a4b4ebc1b227b15))
* **term:** only consider search matches on visible rows when rendering ([008bb23](https://github.com/pmqueiroz/nova/commit/008bb23330fa88292ff78c47268d5f1a2ce6e1f0))
* **term:** resolve theme fonts once per frame instead of per cell ([3954c0c](https://github.com/pmqueiroz/nova/commit/3954c0c8534e61f2a73ac7d3d296fc8dfe4c45d1))
* **ui:** only run the one-second tick while a shell integration is being detected ([8c3e799](https://github.com/pmqueiroz/nova/commit/8c3e7993d7d06d3ffb88886abd41269855c762d5))
* **ui:** only tick cursor blink while a blinking grid cursor is shown ([586fc8b](https://github.com/pmqueiroz/nova/commit/586fc8bffccdbcce51e4a02f26f56298234d04bd))
* **ui:** read the home directory once instead of on every frame ([86ca113](https://github.com/pmqueiroz/nova/commit/86ca113185ac7153c8c95e69b478cb0b573ab8d4))
* **ui:** skip pointer-move updates when nothing tracks the pointer ([49bf345](https://github.com/pmqueiroz/nova/commit/49bf3451bd115c2910aed9420a8c4a3491afed96))
* write settings and command history off the ui thread ([2da978f](https://github.com/pmqueiroz/nova/commit/2da978fbccaf38078c4af3c35c42538a4fff77a2))

## [1.1.1](https://github.com/pmqueiroz/nova/compare/v1.1.0...v1.1.1) (2026-11-02)


### Bug Fixes

* **ai:** skip failure hints that add nothing and name the concrete cause ([3916749](https://github.com/pmqueiroz/nova/commit/39167491b09ee36aaf5b223d4c88735aa98d8420))
* track cwd reported while a command block is still running ([c50a4d9](https://github.com/pmqueiroz/nova/commit/c50a4d9a523db6eb14816fbe1150f6d5788acdd5))
* **update:** swap bundled conpty files during portable updates ([2a7586d](https://github.com/pmqueiroz/nova/commit/2a7586d087b5d1c7c6e1e2a30d320b8d26388c15))
* **windows:** ship openconsole conpty so apps can enable mouse reporting ([ecd8549](https://github.com/pmqueiroz/nova/commit/ecd8549e77ccb73d319b296684fbab09315447fc))

## [1.1.0](https://github.com/pmqueiroz/nova/compare/v1.0.0...v1.1.0) (2026-11-02)


### Features

* **blocks:** take over fullscreen when a command enables bracketed paste ([ffb2593](https://github.com/pmqueiroz/nova/commit/ffb2593e023201078093eaf4acd82817d729f814))
* **cli:** add nova update command to check for new releases ([c58ad85](https://github.com/pmqueiroz/nova/commit/c58ad85c3621b85449373dd81cba8101a2baac29))
* **ds:** match text button height to the segmented control ([d292d05](https://github.com/pmqueiroz/nova/commit/d292d056c0f03a6b9a43fbc225a42a8682acc432))
* **update:** check github for new releases and show an update button ([25a6f8b](https://github.com/pmqueiroz/nova/commit/25a6f8bcb63a6a2753c9d4cb38af59d18698d30e))
* **update:** install updates in place or through the package manager ([d559f98](https://github.com/pmqueiroz/nova/commit/d559f98980beb36a1a6dd45cf37b34bc9742ffd2))


### Bug Fixes

* **blocks:** draw block elements as solid cells in block cards ([503e283](https://github.com/pmqueiroz/nova/commit/503e283ad7553f29c2c53495b1d3f81457a57ca8))
* **blocks:** keep finished fullscreen frames at full width and scroll them sideways while selecting ([b772737](https://github.com/pmqueiroz/nova/commit/b7727377e346cc7568d36ec91842b1ee3559475a))

## [1.0.0](https://github.com/pmqueiroz/nova/compare/v0.24.0...v1.0.0) (2026-11-01)


### Features

* **ai:** rebuild ai sheet with actions and typed answers ([f80a367](https://github.com/pmqueiroz/nova/commit/f80a367a8a76e87ae1ee6773db027dfd39c67ec8))
* **ai:** restyle diagnostic suggestion ([7ff03c3](https://github.com/pmqueiroz/nova/commit/7ff03c3a28153c7917368c3dace9a60041d26b9b))
* **brand:** apply brand-v2 app icons ([7e315e5](https://github.com/pmqueiroz/nova/commit/7e315e5cb48c67f8a2f4d595ecfde3fc3e43b5b8))
* **config:** v2 defaults and keybindings ([97a0f98](https://github.com/pmqueiroz/nova/commit/97a0f98809244c1957577931c655425a01328f26))
* **core:** add block session model ([62cd123](https://github.com/pmqueiroz/nova/commit/62cd123cef74b21fe8d9542f7cb53671cc90d1de))
* **core:** reflow blocks on resize ([aae1df2](https://github.com/pmqueiroz/nova/commit/aae1df21f35eb133925a70d1b452ec6b03baddff))
* **core:** route pty output into blocks ([dd68c2d](https://github.com/pmqueiroz/nova/commit/dd68c2d6da959e4a0a3d365d972bbc6f763960e7))
* **core:** submit commands from nova input ([ad69edf](https://github.com/pmqueiroz/nova/commit/ad69edf8fba9df6d7210e9286acd4f10765b7b2a))
* **pty:** emit block marks from shell integration ([afa8364](https://github.com/pmqueiroz/nova/commit/afa8364eca137061fe006cefc3f84fadbca2b843))
* **settings:** rebuild settings sheet ([c8a9933](https://github.com/pmqueiroz/nova/commit/c8a99333c17493f9643c0e12dad85cf53917a511))
* **settings:** show brand-v2 lockup in about ([d659121](https://github.com/pmqueiroz/nova/commit/d659121d16575b974b3e274b5b0c42850f87d287))
* **theme:** add motion curves and animation helper ([c6e3e4f](https://github.com/pmqueiroz/nova/commit/c6e3e4fdeb34c3fe3b860bcfee23c91c25127f97))
* **title-bar:** shrink the title bar to 48px with small action buttons ([dbf5064](https://github.com/pmqueiroz/nova/commit/dbf5064adb3a66ed0d4e5fd0b7f3a2083e014b68))
* **ui:** add block scroll view ([e8c7295](https://github.com/pmqueiroz/nova/commit/e8c72951dc52ce8d2dd4d2fdab79f77c16530c66))
* **ui:** add ds button ([8c61437](https://github.com/pmqueiroz/nova/commit/8c61437d45d3808d1a21dc5dad00118003e78081))
* **ui:** add ds kbd ([61790d4](https://github.com/pmqueiroz/nova/commit/61790d4445e594d331dbd052346ce05a9f95f90a))
* **ui:** add ds list item ([2a84c19](https://github.com/pmqueiroz/nova/commit/2a84c19a6a8709c5bb2f0ddc44e0f65fea58850a))
* **ui:** add ds press transform ([d99adeb](https://github.com/pmqueiroz/nova/commit/d99adeb988fcbd4a4b0ed6c92302fc0925d07d91))
* **ui:** add ds row ([14c12a0](https://github.com/pmqueiroz/nova/commit/14c12a019485bab3d4f58f8477f6a3de69f18dae))
* **ui:** add ds segmented control ([a8a678a](https://github.com/pmqueiroz/nova/commit/a8a678a8bc2c51e4640d106fd969e0db5e15ab02))
* **ui:** add ds sheet ([3d52773](https://github.com/pmqueiroz/nova/commit/3d52773dd3586e3697e1850503b8b233b7c2ac0e))
* **ui:** add ds stepper ([d317211](https://github.com/pmqueiroz/nova/commit/d31721191f8dde86675342b1a9f9d825b48cb567))
* **ui:** add ds swatch ([3a7794f](https://github.com/pmqueiroz/nova/commit/3a7794faa4e0e55dba2a2a21b4195182d77e645b))
* **ui:** add ds toggle ([99daa0f](https://github.com/pmqueiroz/nova/commit/99daa0f0c23910cbecc0bcd23b8b62638155eb25))
* **ui:** add hover scale to ds transform ([2c34103](https://github.com/pmqueiroz/nova/commit/2c3410326a6f49ea15054753844ee10998a25e7f))
* **ui:** add nova icon set ([cadce6f](https://github.com/pmqueiroz/nova/commit/cadce6f56e2527febb504bec330f0786cf9adc33))
* **ui:** add warp-style input bar ([c7cb8ef](https://github.com/pmqueiroz/nova/commit/c7cb8ef26955fc4def39f31a4205c9af009898fb))
* **ui:** compact layout for narrow windows ([8860dae](https://github.com/pmqueiroz/nova/commit/8860daed5674b013fc49b6851022d0b061335a7a))
* **ui:** drop status bar ([b82c1b1](https://github.com/pmqueiroz/nova/commit/b82c1b1bb75a465151447473ac8d4f4d11bc60fd))
* **ui:** forward keys to running command ([ef05709](https://github.com/pmqueiroz/nova/commit/ef05709045611ea47d914d4ecb6e50884d34bace))
* **ui:** keep the last fullscreen frame in the block ([a465326](https://github.com/pmqueiroz/nova/commit/a4653264b98989b989d0570d43064241f2cbd30a))
* **ui:** map ansi palette to ds colors ([e62bef9](https://github.com/pmqueiroz/nova/commit/e62bef9eca95168336945ec094d98fc7e842929d))
* **ui:** move paste warning to sheet ([6d559dd](https://github.com/pmqueiroz/nova/commit/6d559dd9e787d3d05aef9b790a8518aa136fef7d))
* **ui:** move shell picker to sheet ([f08a14b](https://github.com/pmqueiroz/nova/commit/f08a14b437afc3a1bfebc63bd14eaf727f91382d))
* **ui:** move tabs into title bar ([5383b5c](https://github.com/pmqueiroz/nova/commit/5383b5cc07e9241a3989683dfcbd7060be46dfe2))
* **ui:** raw terminal takeover for full-screen apps ([343e15b](https://github.com/pmqueiroz/nova/commit/343e15b38505b8922f0e6a8681a77be708d0ce0c))
* **ui:** rebuild command palette ([04ce9fa](https://github.com/pmqueiroz/nova/commit/04ce9facd2bac2de45929d581dd2bb1b05a8a7b0))
* **ui:** render command blocks ([76434f1](https://github.com/pmqueiroz/nova/commit/76434f10524daf80b3c0f997a4bb9832cc146804))
* **ui:** restyle search for blocks ([2b2a660](https://github.com/pmqueiroz/nova/commit/2b2a66055f3d7cc81a7f1d64d2d7a2475ad1adac))
* **ui:** restyle split panes ([9106a4c](https://github.com/pmqueiroz/nova/commit/9106a4cfffed05adf48a2d3f15d8b7511b82c807))
* **ui:** restyle title bar ([5334dc5](https://github.com/pmqueiroz/nova/commit/5334dc55facce761f134099bbf28d3285b395211))
* **ui:** restyle window surface ([f8846f9](https://github.com/pmqueiroz/nova/commit/f8846f90f13e529ed0d35d9be00d401328902f2a))
* **ui:** scroll fullscreen programs with the mouse wheel ([5b46d4e](https://github.com/pmqueiroz/nova/commit/5b46d4e8b5447e8e2d7777928be1bafe5d386c53))
* **ui:** select and open links inside blocks ([7aa5254](https://github.com/pmqueiroz/nova/commit/7aa5254337016caa14dc45e34b15cbd4396f6867))
* **ui:** tab completion in the input bar ([4b04afb](https://github.com/pmqueiroz/nova/commit/4b04afb9e20319b17b9072169607497edbda77e4))
* **ui:** v2 global shortcuts ([8974e19](https://github.com/pmqueiroz/nova/commit/8974e197125f5304ebf130d5033ce5cdd37b7bbf))


### Bug Fixes

* **core:** prefix readline submits with kill-line ([ceea08b](https://github.com/pmqueiroz/nova/commit/ceea08b4da0e71d8bbb9bf7a02d3bbdd1b3c4c84))
* **core:** recover block mode after a slow shell start ([3410391](https://github.com/pmqueiroz/nova/commit/341039100b616488182db08ca0ec2f9cd45bdb58))
* **core:** trim blank lines left by block reflow ([d0ca68e](https://github.com/pmqueiroz/nova/commit/d0ca68ee5e4a11d5502d05fd3383b8727c7a6fc4))
* gate windows-only constants to silence dead code on unix ([92c1a99](https://github.com/pmqueiroz/nova/commit/92c1a995d187b135a01c3fdc2ccaf3c84331f190))
* **pty:** keep block marks in order under conpty ([372889a](https://github.com/pmqueiroz/nova/commit/372889aedbea5e925d001d95c08f5f62a061b262))
* **pty:** let pagers use the alternate screen ([fa08b58](https://github.com/pmqueiroz/nova/commit/fa08b58952962f5ad74f7265b8f799ebf7d21e15))
* **pty:** report powershell command-not-found as exit 127 ([00422ae](https://github.com/pmqueiroz/nova/commit/00422ae181b2deba3c80670b23e181a20ac9b5fa))
* **pty:** show a startup error instead of panicking when the program cannot start ([b5db7bb](https://github.com/pmqueiroz/nova/commit/b5db7bba0f7ca5d37a8c88da0a0e1cafe917a059))
* **pty:** use text marks for cmd and git bash under conpty ([c8bdb65](https://github.com/pmqueiroz/nova/commit/c8bdb65457104e2ab2d4453e7362065f4eea3ae9))
* **sheet:** keep a bottom gap matching the top offset ([c7408c0](https://github.com/pmqueiroz/nova/commit/c7408c07109aaf1f6ed4f6c7eae8ddf0f8ac0919))
* **tabs:** stop hidden close icon rendering black on inactive tabs ([682341f](https://github.com/pmqueiroz/nova/commit/682341f9847dc0509997a463e40864a68b3aaf20))
* **terminal:** drop trailing blanks when reflowing lines ([facb81e](https://github.com/pmqueiroz/nova/commit/facb81e755345956567d66cba48df5be2b72d296))
* **terminal:** fill erased cells with the current background color ([fbdd041](https://github.com/pmqueiroz/nova/commit/fbdd041b66a15580504bb900a888755522ba86d7))
* **terminal:** keep the primary screen intact when resizing on the alt screen ([268209f](https://github.com/pmqueiroz/nova/commit/268209f9e98a09528bbfb9b732826756017ffabc))
* **terminal:** parse colon-form truecolor with a color space id ([cc7cca3](https://github.com/pmqueiroz/nova/commit/cc7cca3db645352e56f2ddf1a66ae5c6fb21d749))
* **ui:** draw block elements as solid cell rectangles ([7325729](https://github.com/pmqueiroz/nova/commit/73257297a298bae8ff01e58b85850c565cbe5c1f))
* **ui:** ease the input bar focus ring ([bc156bd](https://github.com/pmqueiroz/nova/commit/bc156bdfb15e078316d88a0e826c2ab833a1547c))
* **ui:** size fullscreen programs to the whole pane ([2d25a8c](https://github.com/pmqueiroz/nova/commit/2d25a8c854efeac3616e26454dd66a976f7ff74f))
* **ui:** stop treating a closed pager pipe as a failure ([fd813d5](https://github.com/pmqueiroz/nova/commit/fd813d547d91e4b41159774f3c382a2487097499))
* **ui:** use iced transparent color for window background ([b56a505](https://github.com/pmqueiroz/nova/commit/b56a50542472dc042e16ecac43b8e7b9bf058519))
* **window:** fit initial window size to the monitor ([5771d30](https://github.com/pmqueiroz/nova/commit/5771d303de9a1571247a218c2b2f4ff4de79e63d))
* **windows:** stop powershell help from blocking on the more pager ([25b7bc1](https://github.com/pmqueiroz/nova/commit/25b7bc19b3c7a37f0d364a5c09ff7f0a16017e65))


### Performance Improvements

* **blocks:** resolve a block's line range once instead of per line ([67e02df](https://github.com/pmqueiroz/nova/commit/67e02dfe133491cebf733af4275ef3fbfbd071f8))
* **settings:** prewarm the installed font list at startup ([4e04215](https://github.com/pmqueiroz/nova/commit/4e04215f6cb77bf719d36d3710e34cc1ee2a5ba6))
* **sheet:** drop the pop scale so text is not re-rasterized every frame ([2175ca6](https://github.com/pmqueiroz/nova/commit/2175ca62eaac25d34ba544ea177db16a7945c743))
* **ui:** only shape and draw visible block output lines ([76420a3](https://github.com/pmqueiroz/nova/commit/76420a3be305e63fed0fb545bc00d42ee1d69196))


### Miscellaneous Chores

* release 1.0.0 ([2cd8695](https://github.com/pmqueiroz/nova/commit/2cd8695c84decff977209c0000815237246a2c52))

## [0.24.0](https://github.com/pmqueiroz/nova/compare/v0.23.2...v0.24.0) (2026-05-19)


### Features

* **settings:** add About tab with brand and version ([4c833a5](https://github.com/pmqueiroz/nova/commit/4c833a51049fb1c9ca274d2c59eb593540bb5462))


### Bug Fixes

* **macos:** prevent premature expansion of NOVA_PROMPT_COMMAND in bash init ([38dfc8c](https://github.com/pmqueiroz/nova/commit/38dfc8cc370303eff579e48353c1a1c246c3a7ca))

## [0.23.2](https://github.com/pmqueiroz/nova/compare/v0.23.1...v0.23.2) (2026-05-18)


### Bug Fixes

* **windows:** start terminal in home dir when launched from install dir ([77ed67c](https://github.com/pmqueiroz/nova/commit/77ed67cc0723283fa4cd85f53a41f5773593b3f4))

## [0.23.1](https://github.com/pmqueiroz/nova/compare/v0.23.0...v0.23.1) (2026-05-18)


### Bug Fixes

* **windows:** restore PATH safety and fix CLI prompt on exit ([5cce647](https://github.com/pmqueiroz/nova/commit/5cce6476752ca06c68f12165a57630c56f99104e))

## [0.23.0](https://github.com/pmqueiroz/nova/compare/v0.22.0...v0.23.0) (2026-05-18)


### Features

* **cli:** add --version / -v flag ([8fe3d7f](https://github.com/pmqueiroz/nova/commit/8fe3d7f77971b4183f26158846cf9f3f795e0eb5))
* **cli:** add config command with view and set subcommands ([0c84a51](https://github.com/pmqueiroz/nova/commit/0c84a516e1118224f0ebf289b65e995d642f4498))


### Bug Fixes

* git bash ps1 prompt and title bar path resolution ([ae3b3ef](https://github.com/pmqueiroz/nova/commit/ae3b3efb5c4caa0cbbf02a7d05601bb48f00ed53))
* **installer:** write PATH to correct registry key for machine-wide installs ([a2010dd](https://github.com/pmqueiroz/nova/commit/a2010dd327646352ed1d780d7e58ed64d3d27904))
* strip trailing semicolon from PROMPT_COMMAND before appending ([9394250](https://github.com/pmqueiroz/nova/commit/9394250441c54e88c7b8aae079ecca817b398315))

## [0.22.0](https://github.com/pmqueiroz/nova/compare/v0.21.1...v0.22.0) (2026-05-18)


### Features

* add grapheme clustering and wide-character support ([048fb96](https://github.com/pmqueiroz/nova/commit/048fb96085a34ba3982a0c2d216328003944a9f0))
* add initial-command and wait-after-command to settings UI ([3c89f78](https://github.com/pmqueiroz/nova/commit/3c89f78d8875a470081969ec01817542898662bb))
* change default paste keybind to ctrl+shift+v on linux/windows ([f693c3f](https://github.com/pmqueiroz/nova/commit/f693c3f7efe4f8d9108e047fa4b811b17b1761ec))
* configurable font family ([fa09c68](https://github.com/pmqueiroz/nova/commit/fa09c68c30a9263a03c42c56a418d8bdffb748de))
* cursor shape variants (beam/block/underline) and blink support ([5cf4f3a](https://github.com/pmqueiroz/nova/commit/5cf4f3a51aebf6312c6968b9a3ba1e3551841ef1))
* font family pick list with all installed fonts ([b3906bb](https://github.com/pmqueiroz/nova/commit/b3906bb7fd7010b9ec659b9328d004f8409cf860))
* implement initial-command and wait-after-command ([ac73a31](https://github.com/pmqueiroz/nova/commit/ac73a31f4df931b7fecea4cba8144a2d7d2c17bd))
* implement kitty keyboard protocol ([9491848](https://github.com/pmqueiroz/nova/commit/94918482edf4bfa0e9aa553082f90c72c35c4a5c))
* implement OSC 52 clipboard read/write support ([285e384](https://github.com/pmqueiroz/nova/commit/285e384cef6857313f5588db6978ff6f70e663a5))
* warn on dangerous paste content ([c7bd8bc](https://github.com/pmqueiroz/nova/commit/c7bd8bc6ca7824218e2eea3e6c2d31775242aa1e))


### Bug Fixes

* initialize split pane git branch on creation ([2647c61](https://github.com/pmqueiroz/nova/commit/2647c61b3563df8537c0dc69dab083cced40ea38))
* **pty:** update title bar and branch on cwd change in linux ([7e0aedf](https://github.com/pmqueiroz/nova/commit/7e0aedfc37fcc622b8c61d2a0b49aea8875c4dbe))
* route paste to active split pane ([5213901](https://github.com/pmqueiroz/nova/commit/52139013f79bb4ec7bb761adcda1f585abcade50))
* suppress unused variable warnings in notification on non-linux targets ([65517eb](https://github.com/pmqueiroz/nova/commit/65517eb066acfaa8475378e3f1ae225f68bc36bb))

## [0.21.1](https://github.com/pmqueiroz/nova/compare/v0.21.0...v0.21.1) (2026-05-17)


### Bug Fixes

* **notification:** guard wait_for_action behind linux-only cfg ([d85bf35](https://github.com/pmqueiroz/nova/commit/d85bf35a25de2a7107ba348ddf4aafa34ed47c81))

## [0.21.0](https://github.com/pmqueiroz/nova/compare/v0.20.0...v0.21.0) (2026-05-17)


### Features

* **notification:** replace shell-spawned with native OS notifications ([ffeee83](https://github.com/pmqueiroz/nova/commit/ffeee83f38f1ac8f00e11580d21063511b59c807))
* **status-bar:** reflect focused split pane state ([698d336](https://github.com/pmqueiroz/nova/commit/698d336e0e6de2596c7e02966d8f5606a10dcd8d))


### Bug Fixes

* **cursor:** hide cursor in TUI ([1611aec](https://github.com/pmqueiroz/nova/commit/1611aec308e225b7a69771be5f8073fdf38bec41))
* **cursor:** remove top-edge resize and fix settings panel passthrough ([70f2965](https://github.com/pmqueiroz/nova/commit/70f29651deadaa279f8f7f2448c60b6bc1eaac49))
* **history:** skip saving input when shell is not at prompt ([c52fb4b](https://github.com/pmqueiroz/nova/commit/c52fb4b24f91f27361a1135b4520fb19f41ca4a7))
* **notification:** gate APP_ID behind windows cfg to silence dead_code on Linux ([f1b337c](https://github.com/pmqueiroz/nova/commit/f1b337c3e2d9feb4abb28409ed213ae91ea0d07f))
* **split-pane:** subtract per-pane padding from each pane's col count ([7277eaa](https://github.com/pmqueiroz/nova/commit/7277eaa8ff32751e1afd61faa887b04057bec434))
* **term:** move cell background to container to eliminate selection gap lines ([945e203](https://github.com/pmqueiroz/nova/commit/945e203e283f3ff308ca8344170ae58e2f6cbbcc))
* **windows:** suppress console flash on startup by adding CREATE_NO_WINDOW to shell detection ([8df3e6a](https://github.com/pmqueiroz/nova/commit/8df3e6a838ba36fb8b1b045fb2b0bbe2bb23f20b))

## [0.20.0](https://github.com/pmqueiroz/nova/compare/v0.19.0...v0.20.0) (2026-05-17)


### Features

* add font size keybinds and debounce font resize for performance ([94967ce](https://github.com/pmqueiroz/nova/commit/94967cee6edd979e2dacdc9f58891cf5e825aaf4))
* close active split pane on Ctrl/Cmd+W instead of entire tab ([24fb272](https://github.com/pmqueiroz/nova/commit/24fb27220cb384c50857e217cd7fb2cd6bd3cb8b))
* command timing ([ae562b5](https://github.com/pmqueiroz/nova/commit/ae562b5bd99eda3ee3bd894c4d4ab47407389086))
* configurable scrollback limit ([a6e6a1c](https://github.com/pmqueiroz/nova/commit/a6e6a1c4dc7ad01fd64db6e42e3e551436bbfce6))
* ctrl-0 reset font size ([878096a](https://github.com/pmqueiroz/nova/commit/878096a9ae8a8744fe4601a02db8d53f98494b55))
* implement split pane with active indicator and pane management ([8bad7d7](https://github.com/pmqueiroz/nova/commit/8bad7d71f9482ec72c8d6a8998a690618321f28c))
* notification on completion ([95820f3](https://github.com/pmqueiroz/nova/commit/95820f3621d3fd88763f37be4eaddfbb035e5fd1))
* resize handle between split panes ([7c8460f](https://github.com/pmqueiroz/nova/commit/7c8460f866393c9d406162024b45b604814edf37))
* search in scrollback ([309e0ed](https://github.com/pmqueiroz/nova/commit/309e0edb210e8b6a365c46dfe2501ac9d28fc4c8))
* tab reordering ([91a9973](https://github.com/pmqueiroz/nova/commit/91a99734b80bd55d53bbfd075d2d25f9ab7071f8))


### Bug Fixes

* address split pane review feedback ([236ef18](https://github.com/pmqueiroz/nova/commit/236ef18ff19d6b8ad8e648beb75c113c11eb8532))
* gate Write import behind windows cfg to fix CI clippy ([038ad34](https://github.com/pmqueiroz/nova/commit/038ad346c0cec860dd34ba82831a37d2afade174))
* prevent evoking diagnostic banner on ctrl c ([c46ab38](https://github.com/pmqueiroz/nova/commit/c46ab3848796ca899a75ed6c295cdd60b2957203))
* search bar layout shift, debounce, accent border, top-right position ([b042831](https://github.com/pmqueiroz/nova/commit/b04283167fb9d7b496fc18052cb0c25bbcc92005))
* use per-cell containers with terminal-level clipping for monospace rendering ([440aa01](https://github.com/pmqueiroz/nova/commit/440aa01e87f5824826c7c1464f9afe87d22a29b2))


### Performance Improvements

* debounce window resize to reduce grid alloc and PTY thrash ([e27f3bc](https://github.com/pmqueiroz/nova/commit/e27f3bca513f1b785879cea65c3f08465afd24cd))

## [0.19.0](https://github.com/pmqueiroz/nova/compare/v0.18.3...v0.19.0) (2026-05-14)


### Features

* **ai:** implement diagnostic banner ([a1cd8b5](https://github.com/pmqueiroz/nova/commit/a1cd8b5cf92bdd97a185066b12293f7585648ed3))
* double-click word and triple-click line selection ([09219b7](https://github.com/pmqueiroz/nova/commit/09219b7786a86690db2aa8bf07a2f94a35d43b00))
* implement command completion suggestions with Tab accept ([a1711c6](https://github.com/pmqueiroz/nova/commit/a1711c66b422d6ec479bbac4e6c08143e5819cd2))
* implement diagnostic banner ui ([4612264](https://github.com/pmqueiroz/nova/commit/4612264ac3ac6673a73cfea3cd5c87c588ef27f9))
* show exit code banner with AI explain suggestion for failed commands ([a8eb44e](https://github.com/pmqueiroz/nova/commit/a8eb44e58661b3ec4e186e2748d21acc0b623b91))


### Bug Fixes

* **macos:** swap arrow key modifier mapping for Option/Cmd ([150ae6e](https://github.com/pmqueiroz/nova/commit/150ae6e11598573822a7ce502ea1b82eb59d41dd))
* remove forced home cwd, open new tabs in focused tab pwd ([43816d5](https://github.com/pmqueiroz/nova/commit/43816d594ca3fdd64e6871452d4b1c6125213841))
* selection now scrolls with scrollback content ([3951a5d](https://github.com/pmqueiroz/nova/commit/3951a5d47852fe0b4891184421741d2d1b51221b))
* selection render and extraction broken during scrollback ([13a30fc](https://github.com/pmqueiroz/nova/commit/13a30fc4840d9f4c876f081c10db08315b2fdc0d))
* suggestion rendering at cursor position with green underline ([d878f1a](https://github.com/pmqueiroz/nova/commit/d878f1aab43f2cca72adb4c39388d6e626e9f097))
* use accent color for shell selection highlights (reverse video + explicit) ([a8408cc](https://github.com/pmqueiroz/nova/commit/a8408ccdfbe07db8a044f2b0b371e68b1f14c94b))

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
