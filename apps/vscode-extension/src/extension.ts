import { translate, translateReverse } from '@braillify/shared'
import * as fs from 'fs'
import * as vscode from 'vscode'

export function activate(context: vscode.ExtensionContext) {
  // 1. 선택 영역 점역 명령어 등록
  const translateCmd = vscode.commands.registerCommand(
    'braillify.translateSelection',
    async () => {
      const editor = vscode.window.activeTextEditor
      if (!editor) return

      const selection = editor.selection
      const text = editor.document.getText(selection)
      if (!text.trim()) {
        vscode.window.showInformationMessage('점역할 텍스트를 선택해 주세요.')
        return
      }

      // 일반 모드로 기본 점역 진행
      const result = await translate(text, 'general')
      if (result.ok) {
        editor.edit((editBuilder) => {
          editBuilder.replace(selection, result.braille)
        })
      } else {
        vscode.window.showErrorMessage(result.error)
      }
    },
  )

  // 2. 선택 영역 역점역 명령어 등록
  const reverseTranslateCmd = vscode.commands.registerCommand(
    'braillify.reverseTranslateSelection',
    async () => {
      const editor = vscode.window.activeTextEditor
      if (!editor) return

      const selection = editor.selection
      const braille = editor.document.getText(selection)
      if (!braille.trim()) {
        vscode.window.showInformationMessage('역점역할 점자를 선택해 주세요.')
        return
      }

      const result = await translateReverse(braille)
      if (result.ok) {
        editor.edit((editBuilder) => {
          editBuilder.replace(selection, result.korean)
        })
      } else {
        vscode.window.showErrorMessage(result.error)
      }
    },
  )

  context.subscriptions.push(translateCmd, reverseTranslateCmd)

  // 3. 사이드바 웹뷰 프로바이더 등록
  const provider = new BraillifyWebviewProvider(context.extensionUri)
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      'braillify.sidebarView',
      provider,
    ),
  )
}

class BraillifyWebviewProvider implements vscode.WebviewViewProvider {
  constructor(private readonly _extensionUri: vscode.Uri) {}

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken,
  ) {
    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [this._extensionUri],
    }

    webviewView.webview.html = this._getHtmlForWebview(webviewView.webview)
  }

  private _getHtmlForWebview(webview: vscode.Webview): string {
    // 빌드된 웹뷰 에셋들의 경로를 맵핑합니다.
    const scriptUri = webview.asWebviewUri(
      vscode.Uri.joinPath(this._extensionUri, 'out', 'webview', 'index.js'),
    )

    // DevupUI는 CSS를 devup-ui.css(테마 변수) + devup-ui-N.css(유틸리티)로 분할
    // 출력하므로, out/webview의 모든 .css 파일을 링크한다. (@layer가 순서를 관리)
    const webviewDir = vscode.Uri.joinPath(this._extensionUri, 'out', 'webview')
    let cssLinks = ''
    try {
      cssLinks = fs
        .readdirSync(webviewDir.fsPath)
        .filter((f) => f.endsWith('.css'))
        .map((f) => {
          const uri = webview.asWebviewUri(
            vscode.Uri.joinPath(webviewDir, f),
          )
          return `<link href="${uri}" rel="stylesheet">`
        })
        .join('\n        ')
    } catch {
      // out/webview가 아직 빌드되지 않은 경우 링크 없이 진행
    }

    return `<!DOCTYPE html>
      <html lang="ko">
      <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        ${cssLinks}
        <title>Braillify</title>
        <style>
          body {
            padding: 0;
            margin: 0;
            background-color: var(--vscode-sideBar-background);
            color: var(--vscode-sideBar-foreground);
            font-family: var(--vscode-font-family);
          }
        </style>
      </head>
      <body>
        <div id="root"></div>
        <script src="${scriptUri}"></script>
      </body>
      </html>`
  }
}

export function deactivate() {}
