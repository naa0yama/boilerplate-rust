# GrapesJS Wireframe Builder

DaisyUI v5 + GrapesJS ワイヤーフレーム環境。

## 起動

devcontainer 起動時に `postStartCommand.sh` が `mise run wireframe:up` を
実行するため、通常は自動でバックグラウンド起動する (ポート `3001` )。
手動での起動/停止は下記コマンドを使う。

```sh
mise run wireframe:up     # バックグラウンド起動 (pidfile: /tmp/wireframe.pid)
mise run wireframe:down   # 停止
# → http://localhost:3001
```

ログは `/tmp/wireframe.log` に出力される。

## エクスポート

ブラウザコンソールで `editor.getHtml()` を実行し、
出力を `tools/wireframe/exports/` に保存（`.gitignore` 対象）。
