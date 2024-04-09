## Website Downloader

このRustライブラリは、ウェブサイトをローカルにダウンロードする機能を提供します。オフラインでのブラウジング、アーカイブ化、またはテストなど、さまざまな目的でウェブサイトのローカルコピーを作成するために使用できます。

### 機能

- ウェブページをダウンロードし、それに関連するCSS、JavaScript、画像などのリソースを含めます。
- ウェブページに埋め込まれた動画のダウンロードをサポートします。
- 相対URLと絶対URLを正しく処理します。
- ネットワークエラー、I/Oエラー、パースエラーなど、さまざまなシナリオのエラーハンドリングを提供します。

### 使用方法

このライブラリを使用したい場合は、提供されているCインターフェースを使用できます。ライブラリは`save_website_extern`関数を公開しており、C++やCバインディングをサポートする他の言語から呼び出すことができます。

以下は、C++から関数を呼び出す方法の例です：

```c++
#include <windows.h>
#include <iostream>

typedef int(*SaveWebsiteExtern)(const char*, const char*);

int main() {
    HINSTANCE hDLL = LoadLibrary(TEXT("website_downloader.dll"));
    if (hDLL == nullptr) {
        std::cerr << "DLL could not be loaded!" << std::endl;
        return 1;
    }

    SaveWebsiteExtern saveWebsite = (SaveWebsiteExtern)GetProcAddress(hDLL, "save_website_extern");
    if (saveWebsite == nullptr) {
        std::cerr << "Function not found in the DLL!" << std::endl;
        FreeLibrary(hDLL);
        return 1;
    }
    int result = saveWebsite("https://example.com","./save/directory");
    if (result == 0) {
        std::cout << "Website saved successfully!" << std::endl;
    }
    else {
        std::cerr << "An error occurred!" << std::endl;
    }

    FreeLibrary(hDLL);
    return 0;
}
```

### エラーハンドリング

このライブラリは、さまざまな失敗シナリオに対する包括的なエラーハンドリングを提供します。エラーはReqwestError、IoError、UrlParseError、SelectorParseErrorなどのタイプに分類されます。これらのタイプに基づいてエラーを適切に処理できます。

### ライセンス

このプロジェクトはMITライセンスのもとで提供されています。詳細は[LICENSE](LICENSE)ファイルを参照してください。

### 貢献

貢献は歓迎します！問題が発生した場合や改善の提案がある場合は、[GitHubリポジトリ](https://github.com/meowkawaiijp/rust_dll_website_copy)でIssueを開いてください。

---

プロジェクトの特定の詳細や要件に応じて、このREADMEをカスタマイズしてください。