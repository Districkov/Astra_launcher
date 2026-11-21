# Локальный запуск: сервер + лончер

Схема та же, что у Majestic: лончер читает манифест, качает ресурсы с CDN,
запускает клиент и подключает его к серверу. У ASTRA всё это уже в коде
(`manifest.rs`, `downloader.rs`, `patcher.rs`, `client.rs`) — не хватало только
манифеста и раздачи.

## 1. Собрать манифест

```powershell
cd C:\Users\Districk\Desktop\1\astra-launcher\tools

# быстрый тест: только скрипты и разметка
python make_manifest.py --src "D:\redage_v3-main\client_packages" `
  --prefix client_resources --version "1.0.0-test" `
  --include ".js,.json,.html,.css" `
  --out "D:\redage_v3-main\client_packages\manifest.json"

# полный: 1910 файлов, 53 ГБ (считает SHA-256, идёт несколько минут)
python make_manifest.py --src "D:\redage_v3-main\client_packages" `
  --prefix client_resources --version "1.0.0" `
  --out "D:\redage_v3-main\client_packages\manifest.json"
```

`--prefix` важен: это и путь в URL, и путь внутри каталога клиента.

## 2. Поднять CDN

```powershell
python serve_local.py --root "D:\redage_v3-main\client_packages" --port 8787 --strip client_resources
```

`--strip` срезает `client_resources/` из URL и отдаёт файл из `client_packages`.
Проверка: <http://127.0.0.1:8787/manifest.json> и
<http://127.0.0.1:8787/client_resources/index.js>.

## 3. Запустить сервер

```powershell
cd D:\redage_v3-main
.\ragemp-server.exe
```

Ждать `Started! Waiting for connections..`. Проверить, что порт 22005 реально
слушается: `Get-NetUDPEndpoint -LocalPort 22005`.

## 4. Запустить лончер

```powershell
C:\Users\Districk\Desktop\1\astra-launcher\src-tauri\target\release\astra-launcher.exe
```

## Конфиг лончера

`%APPDATA%\astra-launcher\config.json` уже переведён на локальный режим,
бэкап прежнего — `config.json.bak-before-local`.

```json
{
  "server_address": "127.0.0.1:22005",
  "connect_mode": "raw",
  "client_manifest_url": "http://127.0.0.1:8787/manifest.json",
  "file_download_base_url": "http://127.0.0.1:8787",
  "rage_dir": "C:\\RAGEMP",
  "gta5_path": "C:\\Program Files (x86)\\Steam\\steamapps\\common\\Grand Theft Auto V"
}
```

Вернуть как было:

```powershell
Copy-Item "$env:APPDATA\astra-launcher\config.json.bak-before-local" `
          "$env:APPDATA\astra-launcher\config.json"
```

## Что мешает на текущем шаге

Сервер поднимается, NeptuneEvo грузится, MySQL и Redis отвечают, в логе
`Started! Waiting for connections..` — но порт 22005 не появляется в таблице
сокетов, UDP-проба получает ICMP port unreachable. То есть клиенту пока
некуда подключаться. Разбираться нужно на живом запуске сервера в обычной
консоли: возможно, порт держит предыдущий процесс, либо RedAge переопределяет
`port` из своего конфига.

## Обход лаунчера RAGE

В лончере это уже реализовано: `LaunchTarget::RagempV` в `patcher.rs` запускает
`ragemp_v.exe` напрямую, минуя `updater.exe` и проверку CDN. Адрес подключения
`ragemp_v.exe` берёт из реестра, а не из своего окна.

Включено: `"launch_target": "ragemp_v"` в `config.json`.

Реестр `HKCU\Software\RAGE-MP` записан (лончер перезаписывает его при каждом Play):

```
launch.ip     = 127.0.0.1
launch.port   = 22005
launch.type   = connect
launch2.ip    = 127.0.0.1
launch2.port  = 22005
```

### Что остаётся

Окно `ragemp_v.exe` всё равно появляется — это и есть клиентский CEF-лаунчер
RAGE. Он подхватывает `launch.type=connect` и уходит на сервер сам, но процесс
и окно никуда не деваются.

Убрать и его можно только одним способом: самим поднимать `GTA5.exe` из папки
Steam и инжектить `multiplayer.dll` через `vinjector.dll` / `loader.dll`, передав
данные подключения. В лончере этого нет — это отдельная работа: надо разобрать,
что именно передаёт `ragemp_v.exe` при инжекте. Плюс в папке `C:\RAGEMP`
`GTA5.exe` и `ragemp_v.exe` — один и тот же файл (21 242 680 Б), то есть
настоящая игра лежит в Steam, а здесь только загрузчик.

Важно: `rui_auto_connect` выключен намеренно. В `patcher.rs` описан эксперимент —
пять вариантов правки `rui\index.rui` (разная длина, отложенный вызов, сохранение
хендшейка `getServers`, правка только `index.html`) ломали клиент: чёрное окно,
`404`, `_rageInitialize is not defined`. Клиент проверяет архив механизмом,
которого в самом файле нет.

