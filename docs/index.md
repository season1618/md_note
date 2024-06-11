# MD Note
Rust製markdown-to-htmlパーサ。
?[](https://github.com/season1618/md_note)

## 使い方
Markdown文書、テンプレートHTML、出力ファイルを用意して実行。
```
$ <md_note> <template>.html <source>.md (<destination>.html)
```

### 属性
md_noteはMarkdown文書から各種データを抽出しテンプレート中の`{属性名}`に埋め込む。利用可能なデータは以下の通り。
- `title`: h1タグ`#`の見出しを文書のタイトルとして用いる。
- `toc`: 文書中の見出しから目次を生成し番号付きリストとして表示。
- `year`, `month`, `day`, `hour`, `minute`, `second`: Markdown文書をHTMLに変換した時刻。
- `content`: 本文。

### テンプレートの例
この文書のテンプレートを示す。
```html
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <link rel="stylesheet" href="./index.css">
  <!-- 省略 -->
  <title>{title}</title>
</head>
<body>
  <nav id="toc">
    <h4>{title}</h4>
    {toc}
  </nav>
  <div id="content">
    <p style="text-align: right;">最終更新: {year}/{month}/{day} {hour}:{minute}:{second}</p>
    {content}
  </div>
</body>
</html>
```

## Markdown
基本的には[CommonMark](https://commonmark.org/help/)。

### リンク
```
[text](url)
```
[MD Note](https://github.com/season1618/md_note)

リンクテキストを省略するとそのURLのページの`<title>`を差し込む。
[](https://github.com/season1618/md_note)

### リンクカード
OGP情報を取得しリンクカードを生成する。
```
?[](url)
```
?[](https://github.com/season1618/md_note)

### 画像
```
![](path)
```
![](./image.jpg)

### 引用
> 行頭に`> `を付けることで引用となる。
> 隣接する行に付けると引用も連結される。

### リスト
- 番号なしリスト
    - `-`
    - `+`
    - `*`

1. 番号付きリスト
    1. `1.`
    1. `2.`
    1. `3.`
    1. ...

- 混合
    1. aaa
        - AAA
    2. bbb
        - BBB
    3. ccc
        - CCC

### 強調
- `*Italic*`: *Italic*
- `_Italic_`: _Italic_
- `**Bold**`: **Bold**
- `__Bold__`: __Bold__

### 数式
HTMLの`<head>`に
```html
<script src="https://polyfill.io/v3/polyfill.min.js?features=es6"></script>
<script id="MathJax-script" async src="https://cdn.jsdelivr.net/npm/mathjax@3/es5/tex-mml-chtml.js"></script>
```
を記述。

- $\psi$: 状態
- $H$: ハミルトニアン
$$
    \psi(t + dt) = \exp\left(-i\frac{H}{\hbar}dt\right)\psi(t)
$$

MathJaxのドキュメントは以下を参照。
?[](https://docs.mathjax.org/en/latest/)

### コード
コードブロックにシンタックスハイライトを付けるには、HTMLの`<head>`に
```html
<link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css">
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js"></script>
<script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/languages/go.min.js"></script>
<script>hljs.highlightAll();</script>
```
を記述。

```c
#include <stdio.h>

int main() {
    printf("hello, world");
    return 0;
}
```

highlight.jsのドキュメントは以下を参照。
?[](https://highlightjs.readthedocs.io/en/latest/)

サポート言語とテーマ。
- [](https://github.com/highlightjs/highlight.js/blob/main/SUPPORTED_LANGUAGES.md)
- [](https://github.com/highlightjs/highlight.js/tree/main/src/styles)
- [](https://cdnjs.com/libraries/highlight.js)

### 表
| aaa | bbb | ccc |
| |
| aaa | bbb | ccc |
| aaa | bbb | ccc |