# Dependency Review Digest
This document consolidates release dependency reviews. Earlier complete snapshots remain available from their signed Git tags and repository history.
Add future release sections here instead of creating another version-named file. Current policies live in the focused documents linked from each release note and in the release roadmap.

## v1.1.2

Complete package-record changes across the four Cargo lockfiles against v1.1.1.
The refreshed native build-script inventory and transport feature checks cover
the bundled AWS-LC update. Saphyr and syn are isolated to maintenance tooling.

| Lockfile | Package | Previous record SHA-256 | Current record SHA-256 |
| --- | --- | --- | --- |
| `Cargo.lock` | `aws-lc-rs` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | `98afc6f2adc77afe214193908c46eb80092ccd7b876750f8362d3444498a043a` |
| `Cargo.lock` | `aws-lc-sys` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | `a28225d127bc54c03e66c384a431c50ccb9dac7f370560a165c958181c089971` |
| `Cargo.lock` | `base64-ng` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | `42abd4af957c0a66e80f4460da967e4e242899929ded2bf975ef97621089409c` |
| `Cargo.lock` | `bumpalo` | `96630bae5e180ab72e28640ae6605a7f6c7eb27250e1856ad18d158ad5020855` | `6490918eb366c002a301a7e9758b8fb6a2d57f510c50e4a03924153cd67b690b` |
| `Cargo.lock` | `bytes` | `2f117edffe9cfb76fe6f5df28a6443dcdcad5a443d77f6bf6eb063b238463c99` | `c620ce74d5087b50f48ec1cec150b3f480a90ca5b1eb5c4c3000acd07186111c` |
| `Cargo.lock` | `cc` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | `925994a9176092b88003eb2d902ca0aac06006d0c5d1b8735b66216cb36484f6` |
| `Cargo.lock` | `cloud-sdk` | `ae63ad64612b9d4257bb2319b354ae03f2121b1072fc477332e82921ffd27f2a` | `76379cdbd6a1b33681187d85911c293f7a1d26f757f6ebe213efcca86a69fc43` |
| `Cargo.lock` | `cloud-sdk-cratesio` | `2207d50f58ea6e0d3a896548ff3a5024a85d68b8f7a344c356e8befd85919de6` | `7e14f2d5be0a58df625a5e078cb22878c50a9ada54d92b7f163d2fe1c5428d9e` |
| `Cargo.lock` | `cloud-sdk-hetzner` | `cf701ff97746d214cff8f78cb5859d1594faeb77bbb5840f085baaaf98632362` | `7c33559d6a8f1734e81fdfbb9278fcdf7754cd00f5dba61f4985177379457e4d` |
| `Cargo.lock` | `cloud-sdk-reqwest` | `daaa49039b2c27da9eeed170ff229279ed02e38c607e8b0af3543ebfbe00aea8` | `0d604477134909440bd17730d11e635b13a555c9be6f7735015fd18672d08c27` |
| `Cargo.lock` | `cloud-sdk-sanitization` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | `5d2bb6ac9bdc5f13342de144b7e77d27cc094cdcd9004022e8f974b7f39020b7` |
| `Cargo.lock` | `cloud-sdk-testkit` | `fba0f3698c07512b25b953254c65d08545ea04b27892e9bcacad5dd7591194c3` | `b2f4cbb770a72c78be61e887f4b6d610ac9b156aa3303428c94c32e51417111e` |
| `Cargo.lock` | `displaydoc` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | `cd14c2070244619a68f1c4eb48350adedab72589df99c15dffd81378bb241f25` |
| `Cargo.lock` | `errno` | `5e945f8e460b12e35afe587a7e7cb692631893796e5346ba62b17d19e08337a4` | `0b1a495fcc18c9e06cfb9b96a0b136cdb047a338fd45ba267d877496eb4d35ca` |
| `Cargo.lock` | `itoa` | `7bc4ba67b30700b0b817c1b51b4519ea60b6047d26aa964b4755544958b6ed85` | `e14b8090f2f138a314ebb98a42830b861eee1bb73a38f3d3afc6cbb63ed36ab3` |
| `Cargo.lock` | `js-sys` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | `13755355a0526660bd132f8df3d277cb1cb9db5d4c1b1677d9c401ea2dbca846` |
| `Cargo.lock` | `log` | `03ff8a5947f163b87ef0fd71740a6ed68927940b3d764d634b5247f071477010` | `390e2ebc9b7737905eab6e93a2a021fb53135da1615529674df914682c1a0aea` |
| `Cargo.lock` | `mio` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | `765df8acc3b55718b40983f8e5d8ad674c2cf3759d8a5597456818d9dace1d58` |
| `Cargo.lock` | `openssl-probe` | `19aaced0c171d00352554958098487ecfdb1d675cc978604e211cb52d3176ede` | `4e2db767526cedd44648dca1d5e4da985dcbdbb1819cd18d3f96125634cbc92f` |
| `Cargo.lock` | `ovhcloud-v2-probe` | `a88f15966e4f57fab4a9ae80cd7df3273799d37e7d327dfaab78c45ee8c6c9d3` | `5ff155402d54fd425c55120715725f0c10d652898b367d8c1326a63f2e091648` |
| `Cargo.lock` | `pin-project-lite` | `c56089249f7ee2b71a32056f802921c3e70229b05a0d4967eed776858897a8ee` | `d1f10a73468eaf88f60b6fa4f7152720b21c4ab35a350ebd3f48155d728f9e24` |
| `Cargo.lock` | `rand` | `5ea1c26ecececa0a0bbc28c8718bb858d34609fe1dbd4c7288e3a201e5edead1` | `c9078cd95c2efa6ee858ff62a2e15028b13d0f2ab160e445b8c8d1c01055d7bb` |
| `Cargo.lock` | `rand_pcg` | `eda88bb1a5f443e685b9cd0465af4d61d00f09aaebab109b5984cf932d15c567` | `efc178f5549a477269e0b55bd8231b184752be5f1e4ab01887d2c7dbd7e09d8c` |
| `Cargo.lock` | `rustix` | `2461dd9ca9585b58fcc5cedae30a70667778fc37fcc19d96daae3a293b19a9fd` | `dbe42b423f5cb044c92db6873bde5752db8ac38e596ae757e8620115ab2e943e` |
| `Cargo.lock` | `rustls-pki-types` | `78c6429dcdd47d7787ca3062cd0b31ab5ea31117db248c65d27ddcabea0d9863` | `f634f81247957cfbad6383bb0cd73ceb34f52ae0b0406324e63c51d5c5cef196` |
| `Cargo.lock` | `rustls-webpki` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | `a0115821156b5cdfe7b9eb11f968a07a556388ed41c34a2c1939ae8b6bf1765f` |
| `Cargo.lock` | `sanitization` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | `e243002e47b30e4ad37b55bcf62e2c1bdfaf7b2caa9370f4c92f5e7049d1954c` |
| `Cargo.lock` | `serde_derive` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | `1cc72675232dcd1a517dbd21f9ccd31dc743f001dd177d1310ca88bbf882a8b6` |
| `Cargo.lock` | `smallvec` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | `39ac4b684c038f0449064be03e759429b862c074d0296877579c922606203ca0` |
| `Cargo.lock` | `syn` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | `4a86a564e6edafd6c64afca414b283fac627e6b083a846ce9ae6e54480e87277` |
| `Cargo.lock` | `synstructure` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | `2161dd5ceb4db724844f0d84c3e93b40a4ef62089c3a5c1d35c86dfd9e2cfa74` |
| `Cargo.lock` | `thiserror-impl` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | `362237c3841e382572b9014e0ca03bf7dfa260cb5002f3f8ea9396c813afb80a` |
| `Cargo.lock` | `utf8_iter` | `ee0319eadd1257fec346b3fb59e12f7cdc38a7a424fd858bee3833dae8d76ea3` | `cae4bef5cbd993119e4a211fa5216a290b4377658191eb421269e0df72a122fb` |
| `Cargo.lock` | `utf_types` | `-` | `b815f7fb53263f42b7e6403a8594ed1b7fec8c38c7cf44e1dc5f8b97201c4480` |
| `Cargo.lock` | `wasm-bindgen` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | `72d541e20fbc953cf9df87810aa2b34237938a48acba5bdd025156570d6c1f84` |
| `Cargo.lock` | `wasm-bindgen-futures` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | `c53628217a955eb7defe1b9370daf8c42ad60821ba649c71d56b7b98bcca079a` |
| `Cargo.lock` | `wasm-bindgen-macro` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | `e6f54b22f2d71692f2509bf928fbba0129889c706b3217bbc078e4018cb3eb89` |
| `Cargo.lock` | `wasm-bindgen-macro-support` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | `6c9d00809063d13bdf337fccd6f025bf96804dc81fea2b3e2cd53ace240246e1` |
| `Cargo.lock` | `wasm-bindgen-shared` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | `3bf62dc594140a70127575f06d978d8126b6db8aebfe5108ff8a6ad1322c99c2` |
| `Cargo.lock` | `web-sys` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | `e4783d35a9e5b3573e53b07c44ca312c792ca524652f747b439cd7d79f7ad0c7` |
| `Cargo.lock` | `webpki-root-certs` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | `c35d59412a87b9cab27a835327467245cae6938b96507e4d9ef0f0ae48aef67a` |
| `Cargo.lock` | `webpki-roots` | `3c788f83f4fb83ff9f27066d0e27ac8f6ec48a88af0ee2a28a1379199d4ab621` | `56b685513f854bc0ae000225bfde7044efe2fa43e0c93ff4484f7bb8ec41d78d` |
| `Cargo.lock` | `yoke-derive` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | `8ff8c99b518cf3fc0a1c6f92584b9304d58511b8343267e6343dfd0a7a8530ed` |
| `Cargo.lock` | `zerofrom-derive` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | `0d1e826a7da730a4b9fc4622b73845d4844235e75bcdde4645b1d13cd60c3f3c` |
| `Cargo.lock` | `zeroize` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | `6983cd0dcb0a4016b91e490b4ea7117fd1b0b2d3cb6386643478562384f61160` |
| `Cargo.lock` | `zerovec-derive` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | `a406f12b48b6c1b371cf53362935a62ace1a9b1b04efbc894526fefb38c20057` |
| `fuzz/Cargo.lock` | `aws-lc-rs` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | `98afc6f2adc77afe214193908c46eb80092ccd7b876750f8362d3444498a043a` |
| `fuzz/Cargo.lock` | `aws-lc-sys` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | `a28225d127bc54c03e66c384a431c50ccb9dac7f370560a165c958181c089971` |
| `fuzz/Cargo.lock` | `base64-ng` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | `42abd4af957c0a66e80f4460da967e4e242899929ded2bf975ef97621089409c` |
| `fuzz/Cargo.lock` | `bumpalo` | `96630bae5e180ab72e28640ae6605a7f6c7eb27250e1856ad18d158ad5020855` | `6490918eb366c002a301a7e9758b8fb6a2d57f510c50e4a03924153cd67b690b` |
| `fuzz/Cargo.lock` | `bytes` | `2f117edffe9cfb76fe6f5df28a6443dcdcad5a443d77f6bf6eb063b238463c99` | `c620ce74d5087b50f48ec1cec150b3f480a90ca5b1eb5c4c3000acd07186111c` |
| `fuzz/Cargo.lock` | `cc` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | `925994a9176092b88003eb2d902ca0aac06006d0c5d1b8735b66216cb36484f6` |
| `fuzz/Cargo.lock` | `cloud-sdk` | `332b282aaaefcae870a0a5ad9f36a09f9518d63e6b0fd9bf9b275ac24803c6ea` | `489e25b2d65cf41f0f72c3b6e9c5b54ea4282a69c1e51371853a1a1d517a435e` |
| `fuzz/Cargo.lock` | `cloud-sdk-cratesio` | `06bf30f096be00144a3d53aefb088db2b4b8e72858a28c5c82d86f97a1a6c2c0` | `479440edb0e76cf9f7695213769829edad1b0af82b7075404f6a44a14039883f` |
| `fuzz/Cargo.lock` | `cloud-sdk-hetzner` | `5505139ad7949155305b88eefdfb103ab61ed761abcd51e2df1948ebd2eb4e53` | `c27caa7d77331b7de6f0c58ae5ef6d0469e9829887ded8a650ef1ca07c6c8db4` |
| `fuzz/Cargo.lock` | `cloud-sdk-reqwest` | `0b4ac09fadf667953df64b8506363736bf8b472b59f80231716bf8d207f045be` | `806f6e36233a70e76218b4200d536f09105de055dc7d024a37f55f890db5c5ae` |
| `fuzz/Cargo.lock` | `cloud-sdk-sanitization` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | `5d2bb6ac9bdc5f13342de144b7e77d27cc094cdcd9004022e8f974b7f39020b7` |
| `fuzz/Cargo.lock` | `displaydoc` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | `cd14c2070244619a68f1c4eb48350adedab72589df99c15dffd81378bb241f25` |
| `fuzz/Cargo.lock` | `itoa` | `7bc4ba67b30700b0b817c1b51b4519ea60b6047d26aa964b4755544958b6ed85` | `e14b8090f2f138a314ebb98a42830b861eee1bb73a38f3d3afc6cbb63ed36ab3` |
| `fuzz/Cargo.lock` | `js-sys` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | `13755355a0526660bd132f8df3d277cb1cb9db5d4c1b1677d9c401ea2dbca846` |
| `fuzz/Cargo.lock` | `log` | `03ff8a5947f163b87ef0fd71740a6ed68927940b3d764d634b5247f071477010` | `390e2ebc9b7737905eab6e93a2a021fb53135da1615529674df914682c1a0aea` |
| `fuzz/Cargo.lock` | `mio` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | `765df8acc3b55718b40983f8e5d8ad674c2cf3759d8a5597456818d9dace1d58` |
| `fuzz/Cargo.lock` | `openssl-probe` | `19aaced0c171d00352554958098487ecfdb1d675cc978604e211cb52d3176ede` | `4e2db767526cedd44648dca1d5e4da985dcbdbb1819cd18d3f96125634cbc92f` |
| `fuzz/Cargo.lock` | `pin-project-lite` | `c56089249f7ee2b71a32056f802921c3e70229b05a0d4967eed776858897a8ee` | `d1f10a73468eaf88f60b6fa4f7152720b21c4ab35a350ebd3f48155d728f9e24` |
| `fuzz/Cargo.lock` | `rand` | `5ea1c26ecececa0a0bbc28c8718bb858d34609fe1dbd4c7288e3a201e5edead1` | `c9078cd95c2efa6ee858ff62a2e15028b13d0f2ab160e445b8c8d1c01055d7bb` |
| `fuzz/Cargo.lock` | `rand_pcg` | `eda88bb1a5f443e685b9cd0465af4d61d00f09aaebab109b5984cf932d15c567` | `efc178f5549a477269e0b55bd8231b184752be5f1e4ab01887d2c7dbd7e09d8c` |
| `fuzz/Cargo.lock` | `rustls-pki-types` | `78c6429dcdd47d7787ca3062cd0b31ab5ea31117db248c65d27ddcabea0d9863` | `f634f81247957cfbad6383bb0cd73ceb34f52ae0b0406324e63c51d5c5cef196` |
| `fuzz/Cargo.lock` | `rustls-webpki` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | `a0115821156b5cdfe7b9eb11f968a07a556388ed41c34a2c1939ae8b6bf1765f` |
| `fuzz/Cargo.lock` | `sanitization` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | `e243002e47b30e4ad37b55bcf62e2c1bdfaf7b2caa9370f4c92f5e7049d1954c` |
| `fuzz/Cargo.lock` | `serde_derive` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | `1cc72675232dcd1a517dbd21f9ccd31dc743f001dd177d1310ca88bbf882a8b6` |
| `fuzz/Cargo.lock` | `smallvec` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | `39ac4b684c038f0449064be03e759429b862c074d0296877579c922606203ca0` |
| `fuzz/Cargo.lock` | `syn` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | `4a86a564e6edafd6c64afca414b283fac627e6b083a846ce9ae6e54480e87277` |
| `fuzz/Cargo.lock` | `synstructure` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | `2161dd5ceb4db724844f0d84c3e93b40a4ef62089c3a5c1d35c86dfd9e2cfa74` |
| `fuzz/Cargo.lock` | `thiserror-impl` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | `362237c3841e382572b9014e0ca03bf7dfa260cb5002f3f8ea9396c813afb80a` |
| `fuzz/Cargo.lock` | `utf8_iter` | `ee0319eadd1257fec346b3fb59e12f7cdc38a7a424fd858bee3833dae8d76ea3` | `cae4bef5cbd993119e4a211fa5216a290b4377658191eb421269e0df72a122fb` |
| `fuzz/Cargo.lock` | `utf_types` | `-` | `b815f7fb53263f42b7e6403a8594ed1b7fec8c38c7cf44e1dc5f8b97201c4480` |
| `fuzz/Cargo.lock` | `wasm-bindgen` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | `72d541e20fbc953cf9df87810aa2b34237938a48acba5bdd025156570d6c1f84` |
| `fuzz/Cargo.lock` | `wasm-bindgen-futures` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | `c53628217a955eb7defe1b9370daf8c42ad60821ba649c71d56b7b98bcca079a` |
| `fuzz/Cargo.lock` | `wasm-bindgen-macro` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | `e6f54b22f2d71692f2509bf928fbba0129889c706b3217bbc078e4018cb3eb89` |
| `fuzz/Cargo.lock` | `wasm-bindgen-macro-support` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | `6c9d00809063d13bdf337fccd6f025bf96804dc81fea2b3e2cd53ace240246e1` |
| `fuzz/Cargo.lock` | `wasm-bindgen-shared` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | `3bf62dc594140a70127575f06d978d8126b6db8aebfe5108ff8a6ad1322c99c2` |
| `fuzz/Cargo.lock` | `web-sys` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | `e4783d35a9e5b3573e53b07c44ca312c792ca524652f747b439cd7d79f7ad0c7` |
| `fuzz/Cargo.lock` | `webpki-root-certs` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | `c35d59412a87b9cab27a835327467245cae6938b96507e4d9ef0f0ae48aef67a` |
| `fuzz/Cargo.lock` | `yoke-derive` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | `8ff8c99b518cf3fc0a1c6f92584b9304d58511b8343267e6343dfd0a7a8530ed` |
| `fuzz/Cargo.lock` | `zerofrom-derive` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | `0d1e826a7da730a4b9fc4622b73845d4844235e75bcdde4645b1d13cd60c3f3c` |
| `fuzz/Cargo.lock` | `zeroize` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | `6983cd0dcb0a4016b91e490b4ea7117fd1b0b2d3cb6386643478562384f61160` |
| `fuzz/Cargo.lock` | `zerovec-derive` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | `a406f12b48b6c1b371cf53362935a62ace1a9b1b04efbc894526fefb38c20057` |
| `tests/reqwest-feature-unification/Cargo.lock` | `async-trait` | `2391db0763ac2d9eaf614322de04ce64862f1efb267909b3c6ba20bf7bd216ca` | `9cd6f3adc55b731ef5c10db95a29f69cffa8773b921d13a438795fae8c55d3dd` |
| `tests/reqwest-feature-unification/Cargo.lock` | `aws-lc-rs` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | `98afc6f2adc77afe214193908c46eb80092ccd7b876750f8362d3444498a043a` |
| `tests/reqwest-feature-unification/Cargo.lock` | `aws-lc-sys` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | `a28225d127bc54c03e66c384a431c50ccb9dac7f370560a165c958181c089971` |
| `tests/reqwest-feature-unification/Cargo.lock` | `base64-ng` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | `42abd4af957c0a66e80f4460da967e4e242899929ded2bf975ef97621089409c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `bumpalo` | `96630bae5e180ab72e28640ae6605a7f6c7eb27250e1856ad18d158ad5020855` | `6490918eb366c002a301a7e9758b8fb6a2d57f510c50e4a03924153cd67b690b` |
| `tests/reqwest-feature-unification/Cargo.lock` | `bytes` | `2f117edffe9cfb76fe6f5df28a6443dcdcad5a443d77f6bf6eb063b238463c99` | `c620ce74d5087b50f48ec1cec150b3f480a90ca5b1eb5c4c3000acd07186111c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `cc` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | `925994a9176092b88003eb2d902ca0aac06006d0c5d1b8735b66216cb36484f6` |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk` | `332b282aaaefcae870a0a5ad9f36a09f9518d63e6b0fd9bf9b275ac24803c6ea` | `489e25b2d65cf41f0f72c3b6e9c5b54ea4282a69c1e51371853a1a1d517a435e` |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk-reqwest` | `9d68f84c05fc79ce5054bf220607444cda47afd17e0db062bff06956ecb45d4a` | `92333ae8f3b47e67c92141e0c06a75541c61200a392fee2885cb9b2881a62638` |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk-sanitization` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | `5d2bb6ac9bdc5f13342de144b7e77d27cc094cdcd9004022e8f974b7f39020b7` |
| `tests/reqwest-feature-unification/Cargo.lock` | `displaydoc` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | `cd14c2070244619a68f1c4eb48350adedab72589df99c15dffd81378bb241f25` |
| `tests/reqwest-feature-unification/Cargo.lock` | `futures-macro` | `b6977608f8a31e605ec45f721b19eaae923826b51d6bdc4898f014f103ea2596` | `2d81472dc886d1c8acb36234bfb634b02cccc60c2b5d55f6f8fe0a7ad13a0ea5` |
| `tests/reqwest-feature-unification/Cargo.lock` | `itoa` | `7bc4ba67b30700b0b817c1b51b4519ea60b6047d26aa964b4755544958b6ed85` | `e14b8090f2f138a314ebb98a42830b861eee1bb73a38f3d3afc6cbb63ed36ab3` |
| `tests/reqwest-feature-unification/Cargo.lock` | `js-sys` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | `13755355a0526660bd132f8df3d277cb1cb9db5d4c1b1677d9c401ea2dbca846` |
| `tests/reqwest-feature-unification/Cargo.lock` | `log` | `03ff8a5947f163b87ef0fd71740a6ed68927940b3d764d634b5247f071477010` | `390e2ebc9b7737905eab6e93a2a021fb53135da1615529674df914682c1a0aea` |
| `tests/reqwest-feature-unification/Cargo.lock` | `mio` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | `765df8acc3b55718b40983f8e5d8ad674c2cf3759d8a5597456818d9dace1d58` |
| `tests/reqwest-feature-unification/Cargo.lock` | `openssl-probe` | `19aaced0c171d00352554958098487ecfdb1d675cc978604e211cb52d3176ede` | `4e2db767526cedd44648dca1d5e4da985dcbdbb1819cd18d3f96125634cbc92f` |
| `tests/reqwest-feature-unification/Cargo.lock` | `pin-project-lite` | `c56089249f7ee2b71a32056f802921c3e70229b05a0d4967eed776858897a8ee` | `d1f10a73468eaf88f60b6fa4f7152720b21c4ab35a350ebd3f48155d728f9e24` |
| `tests/reqwest-feature-unification/Cargo.lock` | `rand` | `5ea1c26ecececa0a0bbc28c8718bb858d34609fe1dbd4c7288e3a201e5edead1` | `c9078cd95c2efa6ee858ff62a2e15028b13d0f2ab160e445b8c8d1c01055d7bb` |
| `tests/reqwest-feature-unification/Cargo.lock` | `rand_pcg` | `eda88bb1a5f443e685b9cd0465af4d61d00f09aaebab109b5984cf932d15c567` | `efc178f5549a477269e0b55bd8231b184752be5f1e4ab01887d2c7dbd7e09d8c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `rustls-pki-types` | `78c6429dcdd47d7787ca3062cd0b31ab5ea31117db248c65d27ddcabea0d9863` | `f634f81247957cfbad6383bb0cd73ceb34f52ae0b0406324e63c51d5c5cef196` |
| `tests/reqwest-feature-unification/Cargo.lock` | `rustls-webpki` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | `a0115821156b5cdfe7b9eb11f968a07a556388ed41c34a2c1939ae8b6bf1765f` |
| `tests/reqwest-feature-unification/Cargo.lock` | `sanitization` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | `e243002e47b30e4ad37b55bcf62e2c1bdfaf7b2caa9370f4c92f5e7049d1954c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `serde_derive` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | `1cc72675232dcd1a517dbd21f9ccd31dc743f001dd177d1310ca88bbf882a8b6` |
| `tests/reqwest-feature-unification/Cargo.lock` | `smallvec` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | `39ac4b684c038f0449064be03e759429b862c074d0296877579c922606203ca0` |
| `tests/reqwest-feature-unification/Cargo.lock` | `syn` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | `4a86a564e6edafd6c64afca414b283fac627e6b083a846ce9ae6e54480e87277` |
| `tests/reqwest-feature-unification/Cargo.lock` | `synstructure` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | `2161dd5ceb4db724844f0d84c3e93b40a4ef62089c3a5c1d35c86dfd9e2cfa74` |
| `tests/reqwest-feature-unification/Cargo.lock` | `thiserror-impl` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | `362237c3841e382572b9014e0ca03bf7dfa260cb5002f3f8ea9396c813afb80a` |
| `tests/reqwest-feature-unification/Cargo.lock` | `tokio-macros` | `95e99e7eb4e2928b04dd4859a733c6d5552d2756dfeec3a254ff9d5cd4f5188c` | `a5dd487adf70b1190ff772595f943ca7d6de405d1552aa790379a90e6983ee5e` |
| `tests/reqwest-feature-unification/Cargo.lock` | `utf8_iter` | `ee0319eadd1257fec346b3fb59e12f7cdc38a7a424fd858bee3833dae8d76ea3` | `cae4bef5cbd993119e4a211fa5216a290b4377658191eb421269e0df72a122fb` |
| `tests/reqwest-feature-unification/Cargo.lock` | `utf_types` | `-` | `b815f7fb53263f42b7e6403a8594ed1b7fec8c38c7cf44e1dc5f8b97201c4480` |
| `tests/reqwest-feature-unification/Cargo.lock` | `uuid` | `a0704088194b1ac4d61e342256d6f71aa9292176218219c5b664d50c2a9f8283` | `96cbd0a884648b470a520925892490b8e9deb3eb58540b0ba02f7978a0aaa99c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | `72d541e20fbc953cf9df87810aa2b34237938a48acba5bdd025156570d6c1f84` |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-futures` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | `c53628217a955eb7defe1b9370daf8c42ad60821ba649c71d56b7b98bcca079a` |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-macro` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | `e6f54b22f2d71692f2509bf928fbba0129889c706b3217bbc078e4018cb3eb89` |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-macro-support` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | `6c9d00809063d13bdf337fccd6f025bf96804dc81fea2b3e2cd53ace240246e1` |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-shared` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | `3bf62dc594140a70127575f06d978d8126b6db8aebfe5108ff8a6ad1322c99c2` |
| `tests/reqwest-feature-unification/Cargo.lock` | `web-sys` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | `e4783d35a9e5b3573e53b07c44ca312c792ca524652f747b439cd7d79f7ad0c7` |
| `tests/reqwest-feature-unification/Cargo.lock` | `webpki-root-certs` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | `c35d59412a87b9cab27a835327467245cae6938b96507e4d9ef0f0ae48aef67a` |
| `tests/reqwest-feature-unification/Cargo.lock` | `webpki-roots` | `3c788f83f4fb83ff9f27066d0e27ac8f6ec48a88af0ee2a28a1379199d4ab621` | `56b685513f854bc0ae000225bfde7044efe2fa43e0c93ff4484f7bb8ec41d78d` |
| `tests/reqwest-feature-unification/Cargo.lock` | `yoke-derive` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | `8ff8c99b518cf3fc0a1c6f92584b9304d58511b8343267e6343dfd0a7a8530ed` |
| `tests/reqwest-feature-unification/Cargo.lock` | `zerofrom-derive` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | `0d1e826a7da730a4b9fc4622b73845d4844235e75bcdde4645b1d13cd60c3f3c` |
| `tests/reqwest-feature-unification/Cargo.lock` | `zeroize` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | `6983cd0dcb0a4016b91e490b4ea7117fd1b0b2d3cb6386643478562384f61160` |
| `tests/reqwest-feature-unification/Cargo.lock` | `zerovec-derive` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | `a406f12b48b6c1b371cf53362935a62ace1a9b1b04efbc894526fefb38c20057` |
| `tools/prepared-coverage-check/Cargo.lock` | `ordered-float` | `c4cb566c72e74bcbd94c074f0eef03a6e15254881ca0085686e33fadd2902dc2` | `2c4d08191d74fe2b60693dee555f3a29d5b0cd8330be9b6588ee182d43eaf8a4` |
| `tools/prepared-coverage-check/Cargo.lock` | `saphyr` | `5648beb030e3ad4d12d82543de1e3997cd2e9c4d8ad83789b633d8cbd7e04edf` | `ff0371b87ee271604d020b451d909236faa1fef4e59b4ad93814667eb9e78ad7` |
| `tools/prepared-coverage-check/Cargo.lock` | `saphyr-parser` | `ae5738276e0675389c1a60ca666f8a922fc88d6dbd48cf91742f6db56c1a3acd` | `0ee604416aeef5eecd114ae2418e68d182abe523cf5f049fd93a391569c9d02a` |
| `tools/prepared-coverage-check/Cargo.lock` | `syn` | `91e9b060694d1c0ab9e50cda54013cdef0d9a8e2b93b6f58738ca14b777a4f76` | `ad912ec2b1758d964302af5e3ee2912e664e753aed72a6c29426839666080e6d` |

## v1.1.1

[Complete review at the signed release](https://github.com/valkyoth/cloud-sdk/blob/v1.1.1/docs/DEPENDENCY_REVIEW.md#v111).

## crates.io Commit 19 Base64 Admission

Reviewed 2026-09-26. The crates.io `alloc` feature admits existing exact default-disabled `base64-ng 2.0.4`; no new registry version, native build or std/runtime feature.
Protected OIDC preflight uses its constant-work URL-safe unpadded scalar decoder with bounded, cleared scratch. JSON parsing is not constant-time; decoding is not signature authentication.
The default no_std graph is unchanged. Only the provider edge is added; default/alloc/MSRV, advisory/license and SBOM checks qualify it. The verifier follow-up belongs to Commit 20.

## crates.io Commit 18 Parser Admission

Reviewed 2026-09-25. Publish metadata promotes existing exact `semver 1.0.28`
from dev-only oracle to optional runtime parser and adds exact `spdx 0.13.5`
with default features disabled. Both edges belong only to the crates.io
provider's opt-in `alloc` feature. Its default graph remains empty of these
parsers, networking, TLS and OS dependencies. SemVer remains MIT/Apache-2.0;
SPDX is Apache-2.0, MSRV 1.85, and supports no_std allocation without its
`std`, `text` or `detection` features. Its sole selected dependency is the
already locked `smallvec 1.16.1`; no new build script or native code is selected.

SPDX registry archive checksum:
`081670c233dfbed55690cc0cd38424e0e24ac1b2673d0b408b3f7b684738dfa9`.
Inputs are public license expressions/requirements bounded to 1024 bytes,
inside the separately bounded metadata parser. Dependency parsing does not
handle credentials. Library parser allocations may abort on OOM; external
resource limits and supervision remain necessary. This avoids inventing an
SPDX expression grammar or replacing the established SemVer parser. The
std-only Cargo platform parser was not admitted; the bounded conservative cfg
profile and its limitations are documented in [the publish policy](CRATESIO_PUBLISH_POLICY.md).
Lockfile/SBOM, license/advisory, isolated alloc/default, MSRV and adversarial
parser checks are required for this checkpoint.

The live direct-pin query also detected `rustls-platform-verifier 0.7.1`.
The existing reviewed `0.7.0` remains pinned in this publication-only checkpoint:
the new patch changes platform hostname normalization and advances the Android
helper to `0.2.0`. Admission/target qualification is explicitly pending in
Commit 20, not silently approved by the new parser admission or represented as
a passing freshness result. The other exact direct pins were current in this
query, including the newly admitted parser.

## v0.24.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.24.0/docs/DEPENDENCY_REVIEW_0.24.0.md)
## v0.32.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.32.0/docs/DEPENDENCY_REVIEW_0.32.0.md)
## v0.33.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.33.0/docs/DEPENDENCY_REVIEW_0.33.0.md)
## v0.34.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.34.0/docs/DEPENDENCY_REVIEW_0.34.0.md)
## v0.35.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.35.0/docs/DEPENDENCY_REVIEW_0.35.0.md)
## v0.36.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.36.0/docs/DEPENDENCY_REVIEW_0.36.0.md)
## v0.37.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.37.0/docs/DEPENDENCY_REVIEW_0.37.0.md)
## v0.38.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.38.0/docs/DEPENDENCY_REVIEW_0.38.0.md)
## v0.39.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.39.0/docs/DEPENDENCY_REVIEW_0.39.0.md)
## v0.40.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.40.0/docs/DEPENDENCY_REVIEW_0.40.0.md)
## v0.41.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.41.0/docs/DEPENDENCY_REVIEW_0.41.0.md)
## v0.42.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.42.0/docs/DEPENDENCY_REVIEW_0.42.0.md)
## v0.43.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.43.0/docs/DEPENDENCY_REVIEW_0.43.0.md)
## v0.44.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.44.0/docs/DEPENDENCY_REVIEW_0.44.0.md)
## v0.45.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.45.0/docs/DEPENDENCY_REVIEW_0.45.0.md)
## v0.46.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.46.0/docs/DEPENDENCY_REVIEW_0.46.0.md)
## v0.47.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.47.0/docs/DEPENDENCY_REVIEW_0.47.0.md)
## v0.48.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.48.0/docs/DEPENDENCY_REVIEW_0.48.0.md)
## v0.49.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.49.0/docs/DEPENDENCY_REVIEW_0.49.0.md)
## v0.50.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.50.0/docs/DEPENDENCY_REVIEW_0.50.0.md)
## v0.51.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.51.0/docs/DEPENDENCY_REVIEW_0.51.0.md)
## v0.52.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.52.0/docs/DEPENDENCY_REVIEW_0.52.0.md)
## v0.53.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.53.0/docs/DEPENDENCY_REVIEW_0.53.0.md)
## v0.54.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.54.0/docs/DEPENDENCY_REVIEW_0.54.0.md)
## v0.55.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.55.0/docs/DEPENDENCY_REVIEW_0.55.0.md)
## v0.56.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.56.0/docs/DEPENDENCY_REVIEW_0.56.0.md)
## v0.57.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.57.0/docs/DEPENDENCY_REVIEW_0.57.0.md)
## v0.58.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.58.0/docs/DEPENDENCY_REVIEW_0.58.0.md)
## v0.59.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.59.0/docs/DEPENDENCY_REVIEW_0.59.0.md)
## v0.60.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.60.0/docs/DEPENDENCY_REVIEW_0.60.0.md)
## v0.61.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.61.0/docs/DEPENDENCY_REVIEW_0.61.0.md)
## v0.62.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.62.0/docs/DEPENDENCY_REVIEW_0.62.0.md)
## v0.63.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.63.0/docs/DEPENDENCY_REVIEW_0.63.0.md)
## v0.64.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.64.0/docs/DEPENDENCY_REVIEW_0.64.0.md)
## v0.65.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.65.0/docs/DEPENDENCY_REVIEW_0.65.0.md)
## v0.66.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.66.0/docs/DEPENDENCY_REVIEW_0.66.0.md)
## v0.67.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.67.0/docs/DEPENDENCY_REVIEW_0.67.0.md)
## v0.68.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.68.0/docs/DEPENDENCY_REVIEW_0.68.0.md)
## v0.69.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.69.0/docs/DEPENDENCY_REVIEW_0.69.0.md)
## v0.70.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.70.0/docs/DEPENDENCY_REVIEW_0.70.0.md)
## v0.71.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.71.0/docs/DEPENDENCY_REVIEW_0.71.0.md)
## v0.72.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.72.0/docs/DEPENDENCY_REVIEW_0.72.0.md)
## v0.73.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.73.0/docs/DEPENDENCY_REVIEW_0.73.0.md)
## v0.74.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.74.0/docs/DEPENDENCY_REVIEW_0.74.0.md)
## v0.75.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.75.0/docs/DEPENDENCY_REVIEW_0.75.0.md)
## v0.76.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.76.0/docs/DEPENDENCY_REVIEW_0.76.0.md)
## v0.77.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.77.0/docs/DEPENDENCY_REVIEW_0.77.0.md)
## v0.78.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.78.0/docs/DEPENDENCY_REVIEW_0.78.0.md)
## v0.79.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.79.0/docs/DEPENDENCY_REVIEW_0.79.0.md)
## v0.80.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.80.0/docs/DEPENDENCY_REVIEW_0.80.0.md)
## v0.81.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.81.0/docs/DEPENDENCY_REVIEW_0.81.0.md)
## v0.82.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.82.0/docs/DEPENDENCY_REVIEW_0.82.0.md)
## v0.83.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.83.0/docs/DEPENDENCY_REVIEW_0.83.0.md)
## v0.84.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.84.0/docs/DEPENDENCY_REVIEW_0.84.0.md)
## v0.85.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.85.0/docs/DEPENDENCY_REVIEW_0.85.0.md)
## v0.86.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.86.0/docs/DEPENDENCY_REVIEW_0.86.0.md)
## v0.87.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.87.0/docs/DEPENDENCY_REVIEW_0.87.0.md)
## v0.88.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.88.0/docs/DEPENDENCY_REVIEW_0.88.0.md)
## v0.89.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.89.0/docs/DEPENDENCY_REVIEW_0.89.0.md)
## v0.90.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.90.0/docs/DEPENDENCY_REVIEW_0.90.0.md)
## v0.91.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.91.0/docs/DEPENDENCY_REVIEW_0.91.0.md)
## v0.92.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.92.0/docs/DEPENDENCY_REVIEW_0.92.0.md)
## v0.93.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.93.0/docs/DEPENDENCY_REVIEW_0.93.0.md)
## v0.94.0
**Full snapshot:** [signed-tag source](https://github.com/valkyoth/cloud-sdk/blob/v0.94.0/docs/DEPENDENCY_REVIEW_0.94.0.md)
## v0.95.0
Status: release candidate; pentest and final retest passed.
### Result
v0.95 adds one exact Unix-only development dependency: `rustix 1.1.4` with
only `fs`, `process`, and `std`. The ignored live integration harness uses its
safe descriptor API for `NOFOLLOW`, `CLOEXEC`, and effective-user ownership
checks. This edge is absent from published normal dependencies, every default
graph, non-Unix targets, and the provider's `no_std` library surface. It adds
no first-party unsafe code, native build, network client, runtime, clock,
randomness, or secret-store edge.
The harness otherwise reuses the already admitted optional reqwest/rustls
Basic transport, first-party sanitization boundary, client workspace, and
strict Robot decoder. The admission rationale and alternatives are recorded in
[`dependency-admission-rustix.md`](dependency-admission-rustix.md).
The optional non-FIPS graph remains exactly `aws-lc-rs 1.18.0`,
`aws-lc-sys 0.44.0`, and `http-body-util 0.1.5`. FIPS packages and features
remain absent and deferred to Brynja. All ordinary first-party crate defaults
remain empty, and `cloud-sdk-hetzner` has no transport dependency.
### Lockfile Changes
| Package | Previous | v0.95 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.94.0` | `0.95.0` | Advance the internal facade for the public Robot checkpoint. |
| `errno` | `-` | `0.3.14` | Target-specific transitive OS-error support for the Unix-only `rustix` development edge. |
| `linux-raw-sys` | `-` | `0.12.1` | Generated Linux syscall constants used by `rustix`; no first-party direct use. |
| `ovhcloud-v2-probe` | `0.94.0` | `0.95.0` | Advance the excluded workspace probe identity only. |
| `rustix` | `-` | `1.1.4` | Exact safe descriptor and effective-user API for the ignored Unix live harness only. |
### Workspace Version Changes
| Package | Published | v0.95 | Change | Publish |
| --- | --- | --- | --- | --- |
| `cloud-sdk` | `0.90.0` | `0.95.0` | cumulative core code | yes |
| `cloud-sdk-hetzner` | `0.45.0` | `0.46.0` | cumulative Robot code and live evidence | yes |
| `cloud-sdk-reqwest` | `0.35.3` | `0.36.0` | accumulated transport code and dependency updates | yes |
| `cloud-sdk-sanitization` | `0.19.0` | `0.19.0` | unchanged | no |
| `cloud-sdk-testkit` | `0.30.5` | `0.31.0` | accumulated regression code | yes |
The release tool must publish exactly the four selected crates in dependency
order and must exclude sanitization, fuzzing, internal tools, isolated tests,
the OVHcloud probe, and retired provider-specific helper crates. Cargo Deny,
RustSec, package, feature-unification, platform, freshness, and complete SPDX
SBOM gates remain mandatory before publication.
## v0.96.0
No dependency or feature edge changed. Lockfile identities advanced
`cloud-sdk` and the unpublished probe from `0.95.0` to `0.96.0`; only
provider request code accumulated and no crate was published.
## v0.97.0
No third-party package, feature, target, build-script, or default edge changed.
Lockfile identities advanced `cloud-sdk` and the unpublished probe from
`0.96.0` to `0.97.0`; metadata and link-local code accumulated for v0.100.
## v0.98.0
No third-party version or feature name changed. `cloud-sdk` and the unpublished
probe advance from `0.97.0` to `0.98.0`. Existing optional reqwest
dependencies move under the exact Linux, Windows, macOS, and FreeBSD target
predicate so unsupported targets reach a crate-owned diagnostic before
third-party compilation. The all-target build-script inventory is pinned;
bundled `aws-lc-sys 0.44.0` remains the admitted native crypto implementation,
conditional `ring 0.17.14` remains reviewed, and FIPS remains absent.
The reqwest `std` marker no longer forces `cloud-sdk/std`; applications needing
the core std integrations enable that core feature explicitly.
| Package | Previous | v0.98 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.97.0` | `0.98.0` | Advance the internal milestone identity. |
| `ovhcloud-v2-probe` | `0.97.0` | `0.98.0` | Advance the unpublished probe identity only. |
## v0.99.0
Exact default-disabled `saphyr 0.0.12` and `saphyr-parser 0.0.12` support the
unpublished bounded YAML workflow checker. They have no published, native,
network, credential, or runtime edge. Nothing publishes at this milestone.
| Package | Previous | v0.99 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.98.0` | `0.99.0` | Advance the internal milestone identity. |
| `ovhcloud-v2-probe` | `0.98.0` | `0.99.0` | Advance the excluded probe identity only. |
| `saphyr` | `-` | `0.0.12` | Exact YAML 1.2 parser in the unpublished workflow-policy checker only; default encoding disabled. |
| `saphyr-parser` | transitive | `0.0.12` | Exact direct pre-DOM event parser; no publishable edge. |
## v0.100.0
No third-party package or feature changed. Credential-free mutation tests are
repository-only and add no runtime, credential, network, build-script, or
package edge. Sanitization remains unchanged; optional AWS-LC remains non-FIPS.
| Package | Previous | v0.100 | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.99.0` | `0.100.0` | Advance the public facade checkpoint. |
| `cloud-sdk-hetzner` | `0.46.0` | `0.47.0` | Publish accumulated request, metadata, and qualification code. |
| `cloud-sdk-reqwest` | `0.36.0` | `0.37.0` | Publish accumulated target and link-local transport code. |
| `cloud-sdk-testkit` | `0.31.0` | `0.31.1` | Dependency-only facade update. |
| `ovhcloud-v2-probe` | `0.99.0` | `0.100.0` | Advance excluded inherited workspace identity only. |
## v1.0.0
No resolved dependency, feature, target, build-script, or default edge changed for the stable tag.
| Package | Previous | Current | Review |
| --- | --- | --- | --- |
| `cloud-sdk` | `0.100.0` | `1.0.0` | Stable promotion; direct third-party requirements become exact. |
| `cloud-sdk-hetzner` | `0.47.0` | `1.0.0` | Stable promotion; direct third-party requirements become exact. |
| `cloud-sdk-reqwest` | `0.37.0` | `1.0.0` | Stable promotion; transport requirements become exact. |
| `cloud-sdk-sanitization` | `0.19.0` | `1.0.0` | Stable promotion; sanitization requirement becomes exact. |
| `cloud-sdk-testkit` | `0.31.1` | `1.0.0` | Stable metadata promotion. |
| `ovhcloud-v2-probe` | `0.100.0` | `1.0.0` | Excluded inherited identity only. |
## v1.1.0

The complete four-lockfile admission and crates.io implementation history remain
in the [published v1.1.0 digest](https://github.com/valkyoth/cloud-sdk/blob/v1.1.0/docs/DEPENDENCY_REVIEW.md#v110).
This bounded digest retains current maintenance evidence above; historical
checksums and acceptance records are preserved at that immutable release tag.
