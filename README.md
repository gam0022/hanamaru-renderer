# hanamaru-renderer
[レイトレ合宿5‽](https://sites.google.com/site/raytracingcamp5/)に向けて開発したRustによるパストレーサーです。

BVHでポリゴンとの衝突判定を高速化したり、薄レンズモデルによる被写界深度を入れたり、IBLしたりしました。

[![test.png](test.png)](test.png)

## Build & Run

[cargo](https://rustup.rs/)をインストールすればすぐにビルド+実行ができます。

```bash
git clone git@github.com:gam0022/hanamaru-renderer.git
cd hanamaru-renderer
cargo run --release
```

4:33以内に自動終了するような設定になっているので、高品質な出力が必要な場合は、[TIME_LIMIT_SEC](https://github.com/gam0022/hanamaru-renderer/blob/master/src/config.rs#L18)の値を大きくしてください。

## 発表スライド
[Hanamaru Renderer for レイトレ合宿5‽](https://speakerdeck.com/gam0022/hanamaru-renderer-for-reitorehe-su-5)