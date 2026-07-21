import type { PluginDefinition } from '@gameap/plugin-sdk';

import ModsTab from './components/ModsTab.vue';

export const goldsrcAddonsPlugin: PluginDefinition = {
    id: 'ezvdsxmlu6fbk',
    name: 'GoldSource Addons',
    version: '0.1.0',
    apiVersion: '1.0',
    description: 'Manage Metamod and AMX Mod X plugins on GoldSource servers',
    author: 'GameAP',

    translations: {
        en: {
            tab_label: 'Plugins',
            'abilities.manage': 'Manage GoldSource addons (Metamod / AMX Mod X)',

            metamod_desc:
                'Modification layer for the GoldSource engine. Loads binary plugins and is required by AMX Mod X.',
            amxx_desc:
                'Scripting platform on top of Metamod: admin tools, statistics and gameplay plugins (.amxx).',
            status_not_installed: 'Not installed',
            status_not_active: 'Not active',
            not_active_hint:
                'The addons directory exists, but liblist.gam does not load Metamod.',
            version_unknown: 'version unknown',
            stats_total: 'Plugins',
            stats_enabled: 'Enabled',
            stats_errors: 'Errors',
            install_hint_metamod:
                'Install Metamod manually into addons/metamod and wire it up in liblist.gam.',
            install_hint_amxx:
                'Install AMX Mod X manually into addons/amxmodx and add it to the Metamod plugins.ini.',

            rcon_unavailable_offline:
                'Server is offline — console versions and statuses are unavailable.',
            rcon_unavailable_norcon:
                'RCON is not configured — console versions and statuses are unavailable.',
            rcon_unavailable_error: 'Failed to query the server console.',
            rcon_unavailable_badpass:
                'Wrong RCON password — console versions and statuses are unavailable. Check the password in the server settings.',
            rcon_unavailable_empty:
                'The server console returned an empty response — versions and statuses are unavailable.',

            restart_required_title: 'Server restart required',
            restart_required_text:
                'Plugin list changes take effect after a server restart or map change.',
            restart_now: 'Restart now',
            restarting: 'Restarting…',
            restart_done: 'Server is restarting',
            restart_failed: 'Failed to restart the server',

            upload_file: 'Upload file',
            search_placeholder: 'Search by name, file, author…',
            filter_all: 'All statuses',
            filter_on: 'Enabled',
            filter_off: 'Disabled',
            filter_err: 'With errors',
            selected: 'Selected: :count',
            bulk_enable: 'Enable',
            bulk_disable: 'Disable',
            bulk_delete: 'Delete',

            col_plugin: 'Plugin',
            col_version: 'Version',
            col_status: 'Status',
            col_enabled: 'On',
            col_actions: 'Actions',

            status_running: 'Running',
            status_enabled: 'Enabled',
            status_paused: 'Paused',
            status_stopped: 'Stopped',
            status_pending: 'Awaiting restart',
            status_error: 'Error',
            status_missing: 'File missing',

            system_badge: 'core',
            system_hint: 'System entry — managed by the platform',
            action_config: 'Config',
            action_source: 'Source',
            action_delete: 'Delete',
            action_pause: 'Pause',
            action_unpause: 'Resume',
            col_debug: 'Debug',
            debug_hint: 'AMX Mod X debug mode for this plugin',
            debug_on: 'Debug enabled for ":name" — applies after restart',
            debug_off: 'Debug disabled for ":name" — applies after restart',
            group_other: 'Other',
            comment_add: 'add comment',
            comment_edit: 'Edit comment',
            comment_placeholder: 'Comment…',
            comment_saved: 'Comment for ":name" saved',

            empty_no_plugins: 'No plugins installed',
            empty_no_results: 'Nothing found — adjust the search',
            install_first: 'Install the first plugin',
            open_in_filemanager: 'open in file manager',

            amxx_missing: 'AMX Mod X is not installed',
            metamod_missing: 'Metamod is not installed',
            platform_missing_hint:
                'Install the platform on the server — the plugin list and file upload will appear here.',
            nothing_installed_title: 'Metamod and AMX Mod X are not installed',
            nothing_installed_text:
                'Install Metamod and AMX Mod X on the server to manage plugins from the panel: enable, disable and upload your own files.',
            not_goldsource:
                'This tab is available only for GoldSource servers (Half-Life 1, CS 1.6).',

            loading: 'Loading…',
            load_failed: 'Failed to load the plugins state',
            retry: 'Retry',

            delete_title: 'Delete plugin ":name"?',
            delete_text_amxx:
                'The plugin file and its plugins.ini line will be removed. Configs are kept.',
            delete_text_metamod:
                'The plugin file and its plugins.ini line will be removed. The directory with configs is kept.',
            bulk_delete_title: 'Delete selected plugins (:count)?',
            bulk_delete_text: 'System entries are not affected.',
            yes: 'Yes',
            no: 'No',

            toggled_on: 'Plugin ":name" enabled — applies after restart',
            toggled_off: 'Plugin ":name" disabled — applies after restart',
            paused_ok: 'Plugin ":name" paused',
            unpaused_ok: 'Plugin ":name" resumed',
            pause_failed: 'Failed to pause ":name"',
            unpause_failed: 'Failed to resume ":name"',
            deleted: 'Plugin ":name" deleted',
            bulk_enabled: 'Plugins enabled: :count',
            bulk_disabled: 'Plugins disabled: :count',
            bulk_deleted: 'Plugins deleted: :count',
            installed_toast: 'Plugin ":name" installed from file',
            op_failed: 'Operation failed',

            install_title_amxx: 'Install plugin — AMX Mod X',
            install_title_metamod: 'Install plugin — Metamod',
            drop_hint: 'Drop a file here or click to choose',
            file_hint_amxx: 'Compiled .amxx files and .sma sources are supported',
            file_hint_metamod: '.so and .dll binaries are supported',
            wrong_type_amxx: 'An .amxx or .sma file is required',
            wrong_type_metamod: 'A .so or .dll file is required',
            auto_enable: 'Enable after install',
            install: 'Install',
            uploading: 'Uploading…',
            overwrite: 'Overwrite',
            overwrite_title: 'Plugin already installed',
            overwrite_text:
                'The existing file will be overwritten with the new version; the plugins.ini entry stays in place.',
            updated_toast: 'Plugin ":name" updated from file',

            config_title: 'Configuration — :name',
            save: 'Save',
            config_saved: 'Configuration saved',
            config_load_failed: 'Failed to load the config',

            source_title: 'Source code — :name',
            source_saved: 'Source saved',
            source_load_failed: 'Failed to load the source',
            compile: 'Compile',
            compiling: 'Compiling…',
            compile_success: 'Compiled successfully — :name updated',
            compile_failed: 'Compilation failed',
            compile_log: 'Compiler output',
            compile_goto_line: 'Go to the error line',
        },
        ru: {
            tab_label: 'Плагины',
            'abilities.manage': 'Управление GoldSource-аддонами (Metamod / AMX Mod X)',

            metamod_desc:
                'Слой модификаций для движка GoldSource. Загружает бинарные плагины и требуется для работы AMX Mod X.',
            amxx_desc:
                'Скриптовая платформа поверх Metamod: администрирование, статистика и геймплейные плагины (.amxx).',
            status_not_installed: 'Не установлен',
            status_not_active: 'Не активен',
            not_active_hint:
                'Каталог найден, но liblist.gam не подключает Metamod.',
            version_unknown: 'версия неизвестна',
            stats_total: 'Плагинов',
            stats_enabled: 'Включено',
            stats_errors: 'Ошибок',
            install_hint_metamod:
                'Установите Metamod вручную в addons/metamod и пропишите его в liblist.gam.',
            install_hint_amxx:
                'Установите AMX Mod X вручную в addons/amxmodx и добавьте его в plugins.ini Metamod.',

            rcon_unavailable_offline:
                'Сервер офлайн — версии и статусы из консоли недоступны.',
            rcon_unavailable_norcon:
                'RCON не настроен — версии и статусы из консоли недоступны.',
            rcon_unavailable_error: 'Не удалось опросить консоль сервера.',
            rcon_unavailable_badpass:
                'Неверный RCON-пароль — версии и статусы из консоли недоступны. Проверьте пароль в настройках сервера.',
            rcon_unavailable_empty:
                'Консоль сервера вернула пустой ответ — версии и статусы недоступны.',

            restart_required_title: 'Требуется перезапуск сервера',
            restart_required_text:
                'Изменения в списке плагинов вступят в силу после перезапуска сервера или смены карты.',
            restart_now: 'Перезапустить сейчас',
            restarting: 'Перезапуск…',
            restart_done: 'Сервер перезапускается',
            restart_failed: 'Не удалось перезапустить сервер',

            upload_file: 'Загрузить файл',
            search_placeholder: 'Поиск по названию, файлу, автору…',
            filter_all: 'Все статусы',
            filter_on: 'Включённые',
            filter_off: 'Выключенные',
            filter_err: 'С ошибками',
            selected: 'Выбрано: :count',
            bulk_enable: 'Включить',
            bulk_disable: 'Выключить',
            bulk_delete: 'Удалить',

            col_plugin: 'Плагин',
            col_version: 'Версия',
            col_status: 'Статус',
            col_enabled: 'Вкл.',
            col_actions: 'Действия',

            status_running: 'Работает',
            status_enabled: 'Включен',
            status_paused: 'На паузе',
            status_stopped: 'Остановлен',
            status_pending: 'Ждёт перезапуска',
            status_error: 'Ошибка',
            status_missing: 'Файл отсутствует',

            system_badge: 'база',
            system_hint: 'Системная запись — управляется платформой',
            action_config: 'Конфиг',
            action_source: 'Исходник',
            action_delete: 'Удалить',
            action_pause: 'Пауза',
            action_unpause: 'Продолжить',
            col_debug: 'Debug',
            debug_hint: 'Режим отладки AMX Mod X для этого плагина',
            debug_on: 'Debug включён для «:name» — применится после перезапуска',
            debug_off: 'Debug выключен для «:name» — применится после перезапуска',
            group_other: 'Прочее',
            comment_add: 'добавить комментарий',
            comment_edit: 'Изменить комментарий',
            comment_placeholder: 'Комментарий…',
            comment_saved: 'Комментарий для «:name» сохранён',

            empty_no_plugins: 'Плагины не установлены',
            empty_no_results: 'Ничего не найдено — измените условия поиска',
            install_first: 'Установить первый плагин',
            open_in_filemanager: 'открыть в файловом менеджере',

            amxx_missing: 'AMX Mod X не установлен',
            metamod_missing: 'Metamod не установлен',
            platform_missing_hint:
                'Установите платформу на сервере — после этого здесь появится список плагинов и загрузка файлов.',
            nothing_installed_title: 'Metamod и AMX Mod X не установлены',
            nothing_installed_text:
                'Установите Metamod и AMX Mod X на сервер — и управляйте плагинами прямо из панели: включение, выключение и загрузка своих файлов.',
            not_goldsource:
                'Вкладка доступна только для серверов на движке GoldSource (Half-Life 1, CS 1.6).',

            loading: 'Загрузка…',
            load_failed: 'Не удалось загрузить состояние плагинов',
            retry: 'Повторить',

            delete_title: 'Удалить плагин «:name»?',
            delete_text_amxx:
                'Файл плагина и строка в plugins.ini будут удалены. Конфигурация сохранится.',
            delete_text_metamod:
                'Файл плагина и строка в plugins.ini будут удалены. Каталог с конфигурацией сохранится.',
            bulk_delete_title: 'Удалить выбранные плагины (:count)?',
            bulk_delete_text: 'Системные записи затронуты не будут.',
            yes: 'Да',
            no: 'Нет',

            toggled_on: 'Плагин «:name» включён — применится после перезапуска',
            toggled_off: 'Плагин «:name» выключен — применится после перезапуска',
            paused_ok: 'Плагин «:name» поставлен на паузу',
            unpaused_ok: 'Плагин «:name» снят с паузы',
            pause_failed: 'Не удалось поставить «:name» на паузу',
            unpause_failed: 'Не удалось снять «:name» с паузы',
            deleted: 'Плагин «:name» удалён',
            bulk_enabled: 'Включено плагинов: :count',
            bulk_disabled: 'Выключено плагинов: :count',
            bulk_deleted: 'Удалено плагинов: :count',
            installed_toast: 'Плагин «:name» установлен из файла',
            op_failed: 'Операция не выполнена',

            install_title_amxx: 'Установка плагина — AMX Mod X',
            install_title_metamod: 'Установка плагина — Metamod',
            drop_hint: 'Перетащите файл сюда или нажмите для выбора',
            file_hint_amxx: 'Поддерживаются готовые .amxx и исходники .sma',
            file_hint_metamod: 'Поддерживаются бинарники .so и .dll',
            wrong_type_amxx: 'Нужен файл .amxx или .sma',
            wrong_type_metamod: 'Нужен файл .so или .dll',
            auto_enable: 'Включить после установки',
            install: 'Установить',
            uploading: 'Загрузка…',
            overwrite: 'Перезаписать',
            overwrite_title: 'Плагин уже установлен',
            overwrite_text:
                'Существующий файл будет перезаписан новой версией; строка в plugins.ini сохранится на своём месте.',
            updated_toast: 'Плагин «:name» обновлён из файла',

            config_title: 'Конфигурация — :name',
            save: 'Сохранить',
            config_saved: 'Конфигурация сохранена',
            config_load_failed: 'Не удалось загрузить конфиг',

            source_title: 'Исходный код — :name',
            source_saved: 'Исходник сохранён',
            source_load_failed: 'Не удалось загрузить исходник',
            compile: 'Скомпилировать',
            compiling: 'Компиляция…',
            compile_success: 'Компиляция успешна — :name обновлён',
            compile_failed: 'Ошибка компиляции',
            compile_log: 'Вывод компилятора',
            compile_goto_line: 'Перейти к строке с ошибкой',
        },
    },

    slots: {
        'server-tabs': [
            {
                component: ModsTab,
                order: 100,
                label: '@:tab_label',
                icon: 'plug',
                name: 'plugins',
                checkPermission: {
                    type: 'hasServerPermissions',
                    permissions: ['plugin:ezvdsxmlu6fbk:manage'],
                },
                checkGame: {
                    engines: ['GoldSource'],
                },
            },
        ],
    },
};
