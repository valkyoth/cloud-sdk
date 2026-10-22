# Dependency Review Digest
This document consolidates release dependency reviews. Earlier complete snapshots remain available from their signed Git tags and repository history.
Add future release sections here instead of creating another version-named file. Current policies live in the focused documents linked from each release note and in the release roadmap.

## v1.1.1

Generated complete-record inventory against v1.1.0 for all four lockfiles.
Transport roots/verifier refresh addresses RUSTSEC-2026-0381/0382.
No new first-party unsafe or default transport activation is authorized.

| Lockfile | Package | Previous record SHA-256 | Current record SHA-256 | Review |
| --- | --- | --- | --- | --- |
| `Cargo.lock` | `aws-lc-rs` | `23e62b2cf4850fbd536bc4556f157ed459df83099dbded859e3d4d9549ca50e0` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `aws-lc-sys` | `ff444e5c40c5edc7003f95b4ba84743267ad1d750c7e4a7e92f458fe351593f5` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `base64-ng` | `985d02463cd495484ee3aff094d4b7a60bdd5e22c5b5f8ddfc9d1455cc1518c4` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cc` | `12c76e53d9eafab8ead36b2759b38a9ba79d4d513d4093029faa487c1a893034` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk` | `7441c5963af8bc2d86e4b322b10e51c9fa85e94da557e119872860d7bf87d8ea` | `ae63ad64612b9d4257bb2319b354ae03f2121b1072fc477332e82921ffd27f2a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk-cratesio` | `bc9a46046c4dfffe20a7d6cd8e68400345a8214cc9b7856d9a08e8fbff5bd1fc` | `2207d50f58ea6e0d3a896548ff3a5024a85d68b8f7a344c356e8befd85919de6` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk-hetzner` | `6122bad0afc7f54a085c63752a2c5f5656813849b0fbf48ef5af2a4dbc69e732` | `cf701ff97746d214cff8f78cb5859d1594faeb77bbb5840f085baaaf98632362` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk-reqwest` | `1817a0e7d11379cf0b35abe8c99834cd6a9920e13d9faf58e07744a3449c556a` | `daaa49039b2c27da9eeed170ff229279ed02e38c607e8b0af3543ebfbe00aea8` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk-sanitization` | `8bad444337eea2ea904886aa754e3ad7aca65dad6518d444e644689a5fa218d2` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `cloud-sdk-testkit` | `ad3525f4799aba3d8e3c49ebbf434beaddd10174d414cd63132d2950039627c5` | `fba0f3698c07512b25b953254c65d08545ea04b27892e9bcacad5dd7591194c3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `combine` | `99ed61e68498904c68d5d320f72a4c41edc84a0bbfeb6574e3b1bc488b44a316` | `9987e44631a4bbd6db6cdbdbb1f7894cb4d0ecb9a3e3bd029f97e4f17239eb4c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `displaydoc` | `74ee4642f694eff272e5033e1b9cdff4685cc75af11f593ede6b73f3d8b53ad9` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `find-msvc-tools` | `a8231f2de5b9aa411e16415c34a91da07ebd2bb3f0d38909e0b53fa7994f8074` | `816b9ab4b7c3bcfba0ec67e5e9abfa489ad45577c532f85e19ad6a719fd3a999` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `hyper` | `9ba77d873483c2ebb9f32c66238f60883f499684bea8597489082306541f4df9` | `7b1770d8086d75e59c2fc1c14068861a7bc4c74db6b289d92dc82bfda8abcd01` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_collections` | `3cc771a37fea92bc028133df7e6a2c585a3213e25da37f495553f3c94185265f` | `139509f958dbee473b53872dc12087c7a9b1bd0a29081a2d064feea87e586fe3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_locale_core` | `1d61630a03ccc2ab1302f2ec8e2d6489d69ad882cc7aa394b80e4793992af5f5` | `876aa7a7f1807aa9a213a3ebefb9929e3670aaa1dfbd2758a44a0a271badd3e9` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_normalizer` | `c637fd905d9ab4cfd46b05c12b2378d40b2195dace32a72161f76ab238124738` | `c1f571243614fddb63a8b0edf8a31367537598e590251d3e8205245563d590f4` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_normalizer_data` | `d1be45b562a650f9ee247c1eb0e0ed4d8ed7eb96e47ea9a30957b189e7647729` | `eb4e4eb3f6955f10784615c5ec324af631ed63a5563f7f1a4ff85e219ad917e7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_properties` | `68812ae540faa6fe5cbf223953ed30da255368b2cb8739c07d404d070eccc889` | `f2c3ac8c8be3011ad5d5b409b793d1173c80cc0fff611cd94488f53034939a26` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_properties_data` | `1d7d397db21c7838c9dee9e8a4a063ebfc9c3ef3ef6813d5ee8b903bd6261b78` | `ae7b3a4420d00db6e7c065d0778c7281c663bdd8b060178cd228f024138d18c3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `icu_provider` | `910ec82a85b4b1c7947c156c3bfaa925d3bbfbbec34cac88729a138f0f19d08b` | `2353093f0e09e1f555454a683a0e9496fbd15ea7d3b5c354ac2c65c16766d21e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `js-sys` | `9f4199a1ae4be3f8e4e356f58f1284a697d5c6f7334d73270e8e4e0e3f90b2eb` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `libc` | `85f1d2ad5a5ef9dd98487258c9e173f0308f3ef8d1844000c5275c3d0cd07d08` | `bcadc804972ac7d62830e7c820799ddcd64526da665ecf587e3a4dfcec14b694` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `mio` | `92870f1b2c416a7be3899dc663d826befa9b4effda0808f53166add637b2d365` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `ovhcloud-v2-probe` | `cb2f873df7bdaafa21085343b4a158c2cee0f88462a50ddf38470621f84a9348` | `a88f15966e4f57fab4a9ae80cd7df3273799d37e7d327dfaab78c45ee8c6c9d3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `quinn-proto` | `18b24cbcd366faa62987037d5ddb9b41a83475e0d54b24d6c6fa7922856b4f9a` | `643856cb0ac5abd515fb0454f80570db9a97c985e2c1643511e646230e09c960` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `quinn-udp` | `c9d5fd2e64c4f913f71936824dcdca832877d0280b95225b608be9b1f54a67bd` | `153d7d9a7832197d37a6b9cd04ffebdf897dea05be73623c644359e6cf257e1f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `rustls` | `8d13cd859b7b28a4e697d196e915f80b0b1c62d2f46015bfe0a0d60f8b250890` | `a95c93f70afc519b67cab70ca1307ba8ce8947b7e8b4db8a63ebd65c8ebb50ca` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `rustls-native-certs` | `966e8d0281d42f35dc5a1aee5bfc37cfd4ab789c9908bf940c3abd4a184b7742` | `63cd3c9dadcccf77964a1bdadf9fabcc3c38aaf465f54fffa362205ffa4af994` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `rustls-webpki` | `bf1dedd4b2031a5ccf373a3bfaa5d256eab4ec4099a034f9fe32fe17ffe375e1` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `sanitization` | `e36f218b7b6168ba74548b8a67d5974abe07fc4a8d7ba98aa7497a312ac264c8` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `schannel` | `a4571acefa904e1ba29776193a3dd33697a0f9cb60dd783ddf1e3c5df6825845` | `57e4f61dad1e8e07a37e83980476363b8de2ce66a092bbf4b4004ea6fd9f9e7e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `serde_derive` | `a30cb17d51ef1b5e6c95d8110c67db1656b64514c320a6d440ab03b165b213eb` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `serde_json` | `08672dc303dacad9f59c9f2b3562f19c078ddff6d12c869852857eee65c5a185` | `2ea66f87f8e73fb34c40f22ebfe15f2e82d2c65cfc48f4a8afde17e52d1d28e7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `sha2` | `f1f556fe72168c29228e501b89efff859310ccf6720da4e641090800cbcac9cc` | `14ad59c60d65b59c89a4fa7c06daee2e25f49db04a58203e2aa1dc439b134f83` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `smallvec` | `cba1ada1b0f899a2acac2ec55c926f2303f361272587c07344efe4c6e7fff87d` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `spdx` | `e0700d2c26cd1af408d92ed97a3510339d488536f77143a982e36edb444e8fb3` | `63ad65ca3e731519e96b8272e1e39ffc76f105ff3f5b2df87bbc4bcb620e9feb` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `syn` | `e84e326fb6ce33f03ca100e700f17303a3316ecebbb1e450de3035e7a7611a80` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `synstructure` | `6c34a34334e5c71eee6241badf9d7224c1d43a77b59c8abe280a2b2aa554cc78` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `thiserror-impl` | `821ff83691ba188adf8f71bb55e542e540edaab71b69ebc105c232bddc06ade4` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `tinystr` | `12d686d7016b06d280b1970201ee73aea35f63b5201ef7aa4e2691e9d8dac047` | `3d83bfc55f71d820f57f00a724636bb68bcfbfd0afe188a721bc3ad062d6daef` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `tokio` | `a5522df2b1abf36cc75e46de770ad5cfbbce47100eea6db31303863c07fe1fda` | `94b59d7e80fedc309217994de51d1dd132e3d05431ad5393fff9f9cc0b44343a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `tokio-rustls` | `66e5bf2fa097f9505ea859dc21c611c9ccf19b0eb8c7bf25c0fd554f9aaa5a74` | `b32897940256bfaab47edfc63fb42dec4f44c540df512711defa572d7783f4c7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `unicode-ident` | `6dd84a51797261e828bc5c48436b57fc95f7c8c73847312b359ca7701b071215` | `ddb8bb94abf9b236c26e41bd4cf88a725c6b8ea85536bd94ba2bd6991f4eba47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `want` | `5b72a1c9f6100d04b2c2d51564dbd1b95541f8c341dc553623caeb2eaa9e99ef` | `f64d80b44beef275b052800dd57649b0d93d5852bef6978908f3db954a4a2918` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `wasm-bindgen` | `8c67a9321f83ba788ba043eab0a9bed228ac36efcdde130d5a5947db9b754599` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `wasm-bindgen-futures` | `fb16e6cef17aeb1ae1ae03cba2dd66a375112fbda421250f0525c217c9d4510f` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `wasm-bindgen-macro` | `f89860d6ff599a1b13a0e850c5743f53d3b5451c72b6eabef14bba7fa754b2da` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `wasm-bindgen-macro-support` | `65c1a50f5ae6dddcb6024d4fdde932c3597a61a7b732f29ac7d39e5ae6a970bd` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `wasm-bindgen-shared` | `5646508e64239dcf5d2b7dcfe1c6ab1a3c9071af80198aaed42cdd49129ca0dd` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `web-sys` | `f125fa1fb8fbf94efad458f4b9709e1b3feab1dd86d817881c815cc99a74d640` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `webpki-root-certs` | `8cd28e023da96410dbc99d9dcc467f4f81a78dbaca525bdd57d23f23a45bb150` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `webpki-roots` | `da03418bc156ddb5fbf3c773366e093bd57bb45ee985ed1f5039ec5b2d07f433` | `3c788f83f4fb83ff9f27066d0e27ac8f6ec48a88af0ee2a28a1379199d4ab621` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `yoke` | `b0fc1b6b627b4b45ded5fd9d327bad08186b72fdee4f7ccf165cf83347ef45bf` | `1cca5279a4c663d851f2e05a0d1674b34aba2a7281b63de869923c79b55ea28f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `yoke-derive` | `054eefa30fdd223359db9460296e661697c466a801c6f52251ea3ccb2d7b5d0a` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `zerofrom-derive` | `b245715e0a2c332d8497040d82f437c746d5eb8623954aac0980f2806d4cc831` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `zeroize` | `1a3c4f789463484fa31842fccf14e24c813297c9c02ccdfac9089b7973ec6b8d` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `zerovec` | `c9c866acce01b0d2440226ae9f79099c53e1fe044d8e64e1a10e3346e432cee5` | `7f91ab7ed49ad52a6cd060fd8bc27b0f212af3d1b8dcccf21ca81f2f88a63ab3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `Cargo.lock` | `zerovec-derive` | `f848b128daa749c5caa9729c8a2640f42867f372cf5de7bf28a5c7db44780568` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `arbitrary` | `cbca28ce955cccb99872766f339d6b0214a53707de13a76281c6ad21f0160e2b` | `94c715916de58d91e1bb3c54de62171b91e1f813058f7eb971de6039b6b9081f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `aws-lc-rs` | `23e62b2cf4850fbd536bc4556f157ed459df83099dbded859e3d4d9549ca50e0` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `aws-lc-sys` | `ff444e5c40c5edc7003f95b4ba84743267ad1d750c7e4a7e92f458fe351593f5` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `base64-ng` | `985d02463cd495484ee3aff094d4b7a60bdd5e22c5b5f8ddfc9d1455cc1518c4` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cc` | `12c76e53d9eafab8ead36b2759b38a9ba79d4d513d4093029faa487c1a893034` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cloud-sdk` | `fb68e1b230c093f915eb1a21df052d43ce12fb3812ec8902929485c4a6fa8cc1` | `332b282aaaefcae870a0a5ad9f36a09f9518d63e6b0fd9bf9b275ac24803c6ea` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cloud-sdk-cratesio` | `f12745db8f47c2eac95617a12d11e098c1b9ca0efdd7ad2eeb23a9e2207c9bd1` | `06bf30f096be00144a3d53aefb088db2b4b8e72858a28c5c82d86f97a1a6c2c0` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cloud-sdk-hetzner` | `2b6c91e3c163ef110840b45a9bb63739124762d0747b7e57c2ed1f42937f7acc` | `5505139ad7949155305b88eefdfb103ab61ed761abcd51e2df1948ebd2eb4e53` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cloud-sdk-reqwest` | `037bae38a101d5d6ba20deca7b6d8f014d7b63a4db4a38e3d743fe73509fc0fb` | `0b4ac09fadf667953df64b8506363736bf8b472b59f80231716bf8d207f045be` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `cloud-sdk-sanitization` | `8bad444337eea2ea904886aa754e3ad7aca65dad6518d444e644689a5fa218d2` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `combine` | `99ed61e68498904c68d5d320f72a4c41edc84a0bbfeb6574e3b1bc488b44a316` | `9987e44631a4bbd6db6cdbdbb1f7894cb4d0ecb9a3e3bd029f97e4f17239eb4c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `displaydoc` | `74ee4642f694eff272e5033e1b9cdff4685cc75af11f593ede6b73f3d8b53ad9` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `find-msvc-tools` | `a8231f2de5b9aa411e16415c34a91da07ebd2bb3f0d38909e0b53fa7994f8074` | `816b9ab4b7c3bcfba0ec67e5e9abfa489ad45577c532f85e19ad6a719fd3a999` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `hyper` | `9ba77d873483c2ebb9f32c66238f60883f499684bea8597489082306541f4df9` | `7b1770d8086d75e59c2fc1c14068861a7bc4c74db6b289d92dc82bfda8abcd01` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_collections` | `3cc771a37fea92bc028133df7e6a2c585a3213e25da37f495553f3c94185265f` | `139509f958dbee473b53872dc12087c7a9b1bd0a29081a2d064feea87e586fe3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_locale_core` | `1d61630a03ccc2ab1302f2ec8e2d6489d69ad882cc7aa394b80e4793992af5f5` | `876aa7a7f1807aa9a213a3ebefb9929e3670aaa1dfbd2758a44a0a271badd3e9` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_normalizer` | `c637fd905d9ab4cfd46b05c12b2378d40b2195dace32a72161f76ab238124738` | `c1f571243614fddb63a8b0edf8a31367537598e590251d3e8205245563d590f4` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_normalizer_data` | `d1be45b562a650f9ee247c1eb0e0ed4d8ed7eb96e47ea9a30957b189e7647729` | `eb4e4eb3f6955f10784615c5ec324af631ed63a5563f7f1a4ff85e219ad917e7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_properties` | `68812ae540faa6fe5cbf223953ed30da255368b2cb8739c07d404d070eccc889` | `f2c3ac8c8be3011ad5d5b409b793d1173c80cc0fff611cd94488f53034939a26` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_properties_data` | `1d7d397db21c7838c9dee9e8a4a063ebfc9c3ef3ef6813d5ee8b903bd6261b78` | `ae7b3a4420d00db6e7c065d0778c7281c663bdd8b060178cd228f024138d18c3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `icu_provider` | `910ec82a85b4b1c7947c156c3bfaa925d3bbfbbec34cac88729a138f0f19d08b` | `2353093f0e09e1f555454a683a0e9496fbd15ea7d3b5c354ac2c65c16766d21e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `js-sys` | `9f4199a1ae4be3f8e4e356f58f1284a697d5c6f7334d73270e8e4e0e3f90b2eb` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `libc` | `85f1d2ad5a5ef9dd98487258c9e173f0308f3ef8d1844000c5275c3d0cd07d08` | `bcadc804972ac7d62830e7c820799ddcd64526da665ecf587e3a4dfcec14b694` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `mio` | `92870f1b2c416a7be3899dc663d826befa9b4effda0808f53166add637b2d365` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `quinn-proto` | `18b24cbcd366faa62987037d5ddb9b41a83475e0d54b24d6c6fa7922856b4f9a` | `643856cb0ac5abd515fb0454f80570db9a97c985e2c1643511e646230e09c960` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `quinn-udp` | `c9d5fd2e64c4f913f71936824dcdca832877d0280b95225b608be9b1f54a67bd` | `153d7d9a7832197d37a6b9cd04ffebdf897dea05be73623c644359e6cf257e1f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `rustls` | `8d13cd859b7b28a4e697d196e915f80b0b1c62d2f46015bfe0a0d60f8b250890` | `a95c93f70afc519b67cab70ca1307ba8ce8947b7e8b4db8a63ebd65c8ebb50ca` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `rustls-native-certs` | `966e8d0281d42f35dc5a1aee5bfc37cfd4ab789c9908bf940c3abd4a184b7742` | `63cd3c9dadcccf77964a1bdadf9fabcc3c38aaf465f54fffa362205ffa4af994` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `rustls-webpki` | `bf1dedd4b2031a5ccf373a3bfaa5d256eab4ec4099a034f9fe32fe17ffe375e1` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `sanitization` | `e36f218b7b6168ba74548b8a67d5974abe07fc4a8d7ba98aa7497a312ac264c8` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `schannel` | `a4571acefa904e1ba29776193a3dd33697a0f9cb60dd783ddf1e3c5df6825845` | `57e4f61dad1e8e07a37e83980476363b8de2ce66a092bbf4b4004ea6fd9f9e7e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `serde_derive` | `a30cb17d51ef1b5e6c95d8110c67db1656b64514c320a6d440ab03b165b213eb` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `serde_json` | `08672dc303dacad9f59c9f2b3562f19c078ddff6d12c869852857eee65c5a185` | `2ea66f87f8e73fb34c40f22ebfe15f2e82d2c65cfc48f4a8afde17e52d1d28e7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `sha2` | `f1f556fe72168c29228e501b89efff859310ccf6720da4e641090800cbcac9cc` | `14ad59c60d65b59c89a4fa7c06daee2e25f49db04a58203e2aa1dc439b134f83` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `smallvec` | `cba1ada1b0f899a2acac2ec55c926f2303f361272587c07344efe4c6e7fff87d` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `spdx` | `e0700d2c26cd1af408d92ed97a3510339d488536f77143a982e36edb444e8fb3` | `63ad65ca3e731519e96b8272e1e39ffc76f105ff3f5b2df87bbc4bcb620e9feb` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `syn` | `e84e326fb6ce33f03ca100e700f17303a3316ecebbb1e450de3035e7a7611a80` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `synstructure` | `6c34a34334e5c71eee6241badf9d7224c1d43a77b59c8abe280a2b2aa554cc78` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `thiserror-impl` | `821ff83691ba188adf8f71bb55e542e540edaab71b69ebc105c232bddc06ade4` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `tinystr` | `12d686d7016b06d280b1970201ee73aea35f63b5201ef7aa4e2691e9d8dac047` | `3d83bfc55f71d820f57f00a724636bb68bcfbfd0afe188a721bc3ad062d6daef` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `tokio` | `a5522df2b1abf36cc75e46de770ad5cfbbce47100eea6db31303863c07fe1fda` | `94b59d7e80fedc309217994de51d1dd132e3d05431ad5393fff9f9cc0b44343a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `tokio-rustls` | `66e5bf2fa097f9505ea859dc21c611c9ccf19b0eb8c7bf25c0fd554f9aaa5a74` | `b32897940256bfaab47edfc63fb42dec4f44c540df512711defa572d7783f4c7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `unicode-ident` | `6dd84a51797261e828bc5c48436b57fc95f7c8c73847312b359ca7701b071215` | `ddb8bb94abf9b236c26e41bd4cf88a725c6b8ea85536bd94ba2bd6991f4eba47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `want` | `5b72a1c9f6100d04b2c2d51564dbd1b95541f8c341dc553623caeb2eaa9e99ef` | `f64d80b44beef275b052800dd57649b0d93d5852bef6978908f3db954a4a2918` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `wasm-bindgen` | `8c67a9321f83ba788ba043eab0a9bed228ac36efcdde130d5a5947db9b754599` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `wasm-bindgen-futures` | `fb16e6cef17aeb1ae1ae03cba2dd66a375112fbda421250f0525c217c9d4510f` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `wasm-bindgen-macro` | `f89860d6ff599a1b13a0e850c5743f53d3b5451c72b6eabef14bba7fa754b2da` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `wasm-bindgen-macro-support` | `65c1a50f5ae6dddcb6024d4fdde932c3597a61a7b732f29ac7d39e5ae6a970bd` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `wasm-bindgen-shared` | `5646508e64239dcf5d2b7dcfe1c6ab1a3c9071af80198aaed42cdd49129ca0dd` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `web-sys` | `f125fa1fb8fbf94efad458f4b9709e1b3feab1dd86d817881c815cc99a74d640` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `webpki-root-certs` | `8cd28e023da96410dbc99d9dcc467f4f81a78dbaca525bdd57d23f23a45bb150` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `yoke` | `b0fc1b6b627b4b45ded5fd9d327bad08186b72fdee4f7ccf165cf83347ef45bf` | `1cca5279a4c663d851f2e05a0d1674b34aba2a7281b63de869923c79b55ea28f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `yoke-derive` | `054eefa30fdd223359db9460296e661697c466a801c6f52251ea3ccb2d7b5d0a` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `zerofrom-derive` | `b245715e0a2c332d8497040d82f437c746d5eb8623954aac0980f2806d4cc831` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `zeroize` | `1a3c4f789463484fa31842fccf14e24c813297c9c02ccdfac9089b7973ec6b8d` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `zerovec` | `c9c866acce01b0d2440226ae9f79099c53e1fe044d8e64e1a10e3346e432cee5` | `7f91ab7ed49ad52a6cd060fd8bc27b0f212af3d1b8dcccf21ca81f2f88a63ab3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `fuzz/Cargo.lock` | `zerovec-derive` | `f848b128daa749c5caa9729c8a2640f42867f372cf5de7bf28a5c7db44780568` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `async-trait` | `4ad7a920c85604d66666cb8141ccf167c26ab06447c3d2d80d3bccd879e79a81` | `2391db0763ac2d9eaf614322de04ce64862f1efb267909b3c6ba20bf7bd216ca` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `aws-lc-rs` | `23e62b2cf4850fbd536bc4556f157ed459df83099dbded859e3d4d9549ca50e0` | `0fef1404c334231c0af78d5c8b31f4ceb82633931e2a36d8d7bcad2c637d4d47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `aws-lc-sys` | `ff444e5c40c5edc7003f95b4ba84743267ad1d750c7e4a7e92f458fe351593f5` | `06ca2a2cf2e136b75572d6f989a5de75b94c57f12ad9ba2e4a0ef2a2f8429497` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `base64-ng` | `985d02463cd495484ee3aff094d4b7a60bdd5e22c5b5f8ddfc9d1455cc1518c4` | `45fe6f4006402e48d029d4e41df25693c26a17f2948a33672387840e3ceec272` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `cc` | `12c76e53d9eafab8ead36b2759b38a9ba79d4d513d4093029faa487c1a893034` | `82139035e0424080177bc9e75c244a1be89f78f6ab191050aeb4d495d2dc5b6c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk` | `fb68e1b230c093f915eb1a21df052d43ce12fb3812ec8902929485c4a6fa8cc1` | `332b282aaaefcae870a0a5ad9f36a09f9518d63e6b0fd9bf9b275ac24803c6ea` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk-reqwest` | `f605eff11670860cb9803e8afd38d8e7e5bf1ed7a49e6fbdf58767dc089b3be9` | `9d68f84c05fc79ce5054bf220607444cda47afd17e0db062bff06956ecb45d4a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `cloud-sdk-sanitization` | `8bad444337eea2ea904886aa754e3ad7aca65dad6518d444e644689a5fa218d2` | `9a46dc03b2a695d4aa2896c4e5edd87301d97b54c33c3dc1ff4555f1fdd29885` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `combine` | `99ed61e68498904c68d5d320f72a4c41edc84a0bbfeb6574e3b1bc488b44a316` | `9987e44631a4bbd6db6cdbdbb1f7894cb4d0ecb9a3e3bd029f97e4f17239eb4c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `displaydoc` | `74ee4642f694eff272e5033e1b9cdff4685cc75af11f593ede6b73f3d8b53ad9` | `3c74f60fe48c0b88d964f822f16b8c19803d844d9a2b530ed9882b8b4c34ddb6` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `either` | `88b14b39a61b0ce50ebfa8ec91ee299ef8e3c06c0f3c2744a0237c8ce0f9ee6a` | `d770f588c1a54ad8e15d79dae9bf9d895de478604f1c1021216e9983f0774f8a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `equivalent` | `9463296fbd750a4406a8d3c092278ad0594055f133c34f159f331de604f40f6d` | `a4b140657dfbad40a39cdcf73b236672b14fcc6e5a14b169eaa5499288553b48` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `find-msvc-tools` | `a8231f2de5b9aa411e16415c34a91da07ebd2bb3f0d38909e0b53fa7994f8074` | `816b9ab4b7c3bcfba0ec67e5e9abfa489ad45577c532f85e19ad6a719fd3a999` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `futures-macro` | `81e1ad315795b3a6994fd3218f9beb671671acb6f2e420ac94c3a939a7fc7351` | `b6977608f8a31e605ec45f721b19eaae923826b51d6bdc4898f014f103ea2596` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `h2` | `c8ed309045d9e5bc2953b9b22f07976ed95666c14483c7c159b499a0d510f7c0` | `8c94ff414dcbdd2693fcc43c2789f2533001eaca0ae22a18927900a239d0375c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `hyper` | `0ea822ef3be75b4fef34851af1e47d2564c3976e94c186b588c36a786c2c1912` | `9682a247169dcc4dc339972a7255f1678c5836b1b25e3d45e48237b620fd63d6` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_collections` | `3cc771a37fea92bc028133df7e6a2c585a3213e25da37f495553f3c94185265f` | `139509f958dbee473b53872dc12087c7a9b1bd0a29081a2d064feea87e586fe3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_locale_core` | `1d61630a03ccc2ab1302f2ec8e2d6489d69ad882cc7aa394b80e4793992af5f5` | `876aa7a7f1807aa9a213a3ebefb9929e3670aaa1dfbd2758a44a0a271badd3e9` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_normalizer` | `c637fd905d9ab4cfd46b05c12b2378d40b2195dace32a72161f76ab238124738` | `c1f571243614fddb63a8b0edf8a31367537598e590251d3e8205245563d590f4` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_normalizer_data` | `d1be45b562a650f9ee247c1eb0e0ed4d8ed7eb96e47ea9a30957b189e7647729` | `eb4e4eb3f6955f10784615c5ec324af631ed63a5563f7f1a4ff85e219ad917e7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_properties` | `68812ae540faa6fe5cbf223953ed30da255368b2cb8739c07d404d070eccc889` | `f2c3ac8c8be3011ad5d5b409b793d1173c80cc0fff611cd94488f53034939a26` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_properties_data` | `1d7d397db21c7838c9dee9e8a4a063ebfc9c3ef3ef6813d5ee8b903bd6261b78` | `ae7b3a4420d00db6e7c065d0778c7281c663bdd8b060178cd228f024138d18c3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `icu_provider` | `910ec82a85b4b1c7947c156c3bfaa925d3bbfbbec34cac88729a138f0f19d08b` | `2353093f0e09e1f555454a683a0e9496fbd15ea7d3b5c354ac2c65c16766d21e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `js-sys` | `9f4199a1ae4be3f8e4e356f58f1284a697d5c6f7334d73270e8e4e0e3f90b2eb` | `e7edf9f830a0a2fdcb32431b8970705c95f47a603e1831267353ddfab3ce2ca5` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `libc` | `85f1d2ad5a5ef9dd98487258c9e173f0308f3ef8d1844000c5275c3d0cd07d08` | `bcadc804972ac7d62830e7c820799ddcd64526da665ecf587e3a4dfcec14b694` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `mio` | `92870f1b2c416a7be3899dc663d826befa9b4effda0808f53166add637b2d365` | `34944a6f6219f47bd981c9427f2aa9934c5a218bd878e079bafb67ae40dd3608` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `quinn-proto` | `18b24cbcd366faa62987037d5ddb9b41a83475e0d54b24d6c6fa7922856b4f9a` | `643856cb0ac5abd515fb0454f80570db9a97c985e2c1643511e646230e09c960` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `quinn-udp` | `c9d5fd2e64c4f913f71936824dcdca832877d0280b95225b608be9b1f54a67bd` | `153d7d9a7832197d37a6b9cd04ffebdf897dea05be73623c644359e6cf257e1f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `rustls` | `8d13cd859b7b28a4e697d196e915f80b0b1c62d2f46015bfe0a0d60f8b250890` | `a95c93f70afc519b67cab70ca1307ba8ce8947b7e8b4db8a63ebd65c8ebb50ca` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `rustls-native-certs` | `966e8d0281d42f35dc5a1aee5bfc37cfd4ab789c9908bf940c3abd4a184b7742` | `63cd3c9dadcccf77964a1bdadf9fabcc3c38aaf465f54fffa362205ffa4af994` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `rustls-webpki` | `bf1dedd4b2031a5ccf373a3bfaa5d256eab4ec4099a034f9fe32fe17ffe375e1` | `59300b38975a620c7d059f1c85f6a24dfed822127a6e92b1744a3fb7cc1b361f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `sanitization` | `e36f218b7b6168ba74548b8a67d5974abe07fc4a8d7ba98aa7497a312ac264c8` | `655839959f67b38140c0d2159fcb8aff4abe57271a294ae9083c62bf121eab3e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `schannel` | `a4571acefa904e1ba29776193a3dd33697a0f9cb60dd783ddf1e3c5df6825845` | `57e4f61dad1e8e07a37e83980476363b8de2ce66a092bbf4b4004ea6fd9f9e7e` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `serde_derive` | `a30cb17d51ef1b5e6c95d8110c67db1656b64514c320a6d440ab03b165b213eb` | `082ffacfd4d5939fcca96edb937c29eba5eb0c3ecf0bfbda7b350270ab7978ff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `smallvec` | `cba1ada1b0f899a2acac2ec55c926f2303f361272587c07344efe4c6e7fff87d` | `476c4fc43e6a25f3961a66d7840810cf43cbb862dc13bac367f9b571e651181a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `syn` | `e84e326fb6ce33f03ca100e700f17303a3316ecebbb1e450de3035e7a7611a80` | `3a569f081be7416af3df57328be1ff4d146c5f4c26df226a6849ac3f44a9d70c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `synstructure` | `6c34a34334e5c71eee6241badf9d7224c1d43a77b59c8abe280a2b2aa554cc78` | `379c7cbe1612f565297503726e0e3f7c2a4d5808b171f2bb4fc25cb66197bbff` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `thiserror-impl` | `821ff83691ba188adf8f71bb55e542e540edaab71b69ebc105c232bddc06ade4` | `d3c0acd136f5ffb02dc5912c4e34f9f9726beeda4bd07460f59dddb043b73149` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `tinystr` | `12d686d7016b06d280b1970201ee73aea35f63b5201ef7aa4e2691e9d8dac047` | `3d83bfc55f71d820f57f00a724636bb68bcfbfd0afe188a721bc3ad062d6daef` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `tokio` | `153d9028443c1cd91cd51e1e178cce7ea802013c0a8a6894aa3813a0f28a3a80` | `dbd092a0663ce03132713f55556523f4fba38f1bad6f53102fcafb4c7b0d4615` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `tokio-macros` | `beff217ae2564fdf0ead50e1f0076485e36026fe86e453c98da6185779362ab8` | `95e99e7eb4e2928b04dd4859a733c6d5552d2756dfeec3a254ff9d5cd4f5188c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `tokio-rustls` | `66e5bf2fa097f9505ea859dc21c611c9ccf19b0eb8c7bf25c0fd554f9aaa5a74` | `b32897940256bfaab47edfc63fb42dec4f44c540df512711defa572d7783f4c7` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `tokio-util` | `61c04f9b7e8a11fcb23fee18010deb62af865b0b4b580d4ad028a10e267ebb89` | `974e96192dd7bc85037bc90b678e1b6fa035f34c5c25add9b5628af90fa4a9e5` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `unicode-ident` | `6dd84a51797261e828bc5c48436b57fc95f7c8c73847312b359ca7701b071215` | `ddb8bb94abf9b236c26e41bd4cf88a725c6b8ea85536bd94ba2bd6991f4eba47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `uuid` | `67e2c527d5348e50d1a5c37c9c0facba62b425531f2297fe0b1b2ef0baf5c3ab` | `a0704088194b1ac4d61e342256d6f71aa9292176218219c5b664d50c2a9f8283` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `want` | `5b72a1c9f6100d04b2c2d51564dbd1b95541f8c341dc553623caeb2eaa9e99ef` | `f64d80b44beef275b052800dd57649b0d93d5852bef6978908f3db954a4a2918` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen` | `8c67a9321f83ba788ba043eab0a9bed228ac36efcdde130d5a5947db9b754599` | `dee49676ebc94a641907795ba36ba07943c5f31de8d3574b8e964fb4669da643` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-futures` | `fb16e6cef17aeb1ae1ae03cba2dd66a375112fbda421250f0525c217c9d4510f` | `ecaa5fadcff5e3829fb62199623ca333d857d99effbb294548fc1db5e5cf879c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-macro` | `f89860d6ff599a1b13a0e850c5743f53d3b5451c72b6eabef14bba7fa754b2da` | `e06ed827a07db83715e36c446be7ad8b3f6b358f6ef3fb57e3315dd188d81e5c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-macro-support` | `65c1a50f5ae6dddcb6024d4fdde932c3597a61a7b732f29ac7d39e5ae6a970bd` | `797500f742111a1143e0e21b3d34f1b84aea7685618753bc0b17709fc967f266` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `wasm-bindgen-shared` | `5646508e64239dcf5d2b7dcfe1c6ab1a3c9071af80198aaed42cdd49129ca0dd` | `5cbe316a3a4f2efbb0e6d4999bdc1c560f451940a2c62b1f39d5fa76027e0b91` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `web-sys` | `f125fa1fb8fbf94efad458f4b9709e1b3feab1dd86d817881c815cc99a74d640` | `80f6d5cd07039beca20bb92420673db32d64f915fde938f9acadbf1919fe5a21` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `webpki-root-certs` | `8cd28e023da96410dbc99d9dcc467f4f81a78dbaca525bdd57d23f23a45bb150` | `7d22caf9638d147be9dad242b867223d250145d09be8d5c9d24b7dbc434ed21a` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `webpki-roots` | `da03418bc156ddb5fbf3c773366e093bd57bb45ee985ed1f5039ec5b2d07f433` | `3c788f83f4fb83ff9f27066d0e27ac8f6ec48a88af0ee2a28a1379199d4ab621` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `yoke` | `b0fc1b6b627b4b45ded5fd9d327bad08186b72fdee4f7ccf165cf83347ef45bf` | `1cca5279a4c663d851f2e05a0d1674b34aba2a7281b63de869923c79b55ea28f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `yoke-derive` | `054eefa30fdd223359db9460296e661697c466a801c6f52251ea3ccb2d7b5d0a` | `c4bf1dd542e75d22d9f2612d0dfd0bad11f7ac129505e502d07fb471be3cd93f` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `zerofrom-derive` | `b245715e0a2c332d8497040d82f437c746d5eb8623954aac0980f2806d4cc831` | `9043f0d1fbec2cc5ccee878759ed6b2ef8aff39d4c0b715aded238d9ea51be67` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `zeroize` | `1a3c4f789463484fa31842fccf14e24c813297c9c02ccdfac9089b7973ec6b8d` | `5f686ff103d4f80a1fc7ecf160e22a7748ab11e78fe2e6ed36477e49431160a2` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `zerovec` | `c9c866acce01b0d2440226ae9f79099c53e1fe044d8e64e1a10e3346e432cee5` | `7f91ab7ed49ad52a6cd060fd8bc27b0f212af3d1b8dcccf21ca81f2f88a63ab3` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tests/reqwest-feature-unification/Cargo.lock` | `zerovec-derive` | `f848b128daa749c5caa9729c8a2640f42867f372cf5de7bf28a5c7db44780568` | `f2db69f773df89ec19f4743808bedc442b409a7b1edfefb73b987388577b4f7c` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tools/prepared-coverage-check/Cargo.lock` | `saphyr` | `492be9e4562ba9213479eb548a70baca4e46d764ea1e387b17f33bca6c3e7541` | `5648beb030e3ad4d12d82543de1e3997cd2e9c4d8ad83789b633d8cbd7e04edf` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tools/prepared-coverage-check/Cargo.lock` | `saphyr-parser` | `e7603eee685fe9cf15d38cb0d29ae36b90f0f3aa5834a0c387ee9b4baadd9e08` | `ae5738276e0675389c1a60ca666f8a922fc88d6dbd48cf91742f6db56c1a3acd` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tools/prepared-coverage-check/Cargo.lock` | `syn` | `4946a797613cd574557b6b00c8c20910f058bb5e164ae3939da3c94800bc32d6` | `91e9b060694d1c0ab9e50cda54013cdef0d9a8e2b93b6f58738ca14b777a4f76` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |
| `tools/prepared-coverage-check/Cargo.lock` | `unicode-ident` | `6dd84a51797261e828bc5c48436b57fc95f7c8c73847312b359ca7701b071215` | `ddb8bb94abf9b236c26e41bd4cf88a725c6b8ea85536bd94ba2bd6991f4eba47` | Maintenance refresh; retained feature boundaries; validation recorded in release notes. |

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
