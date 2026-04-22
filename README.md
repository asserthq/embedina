## Assistant

Вот обзор текущего дерева проекта, комментарии по тому, что в нём находится, и практичные рекомендации — куда положить что, чтобы `embedina` оставался общей инфраструктурой, а прототипы жили в подпроектах/examples. Внизу — конкретные команды для реорганизации (создать подпроекты, превратить `src/adcs` в отдельный crate, обновить workspace и т.д.).

1) Текущее дерево (главные элементы)
- В корне:
  - `Cargo.toml` — корневой манифест (пока содержит `[[bin]]` для `adcs` и зависимости).
  - `.cargo/config.toml` — runner/target/rustflags для probe-rs + flip-link.
  - `README.md`, лицензии и настройки редактора.
  - `examples/` — демонстрационные примеры (`hello`, `levels`, `bitfield`, ...).
  - `src/` — содержит `src/lib.rs` (общая библиотека `embedina`) и `src/adcs/main.rs` (сейчас это отдельный бинарник в `src`).
  - `tests/` — интеграционные тесты.

Конкретно — см. `Cargo.toml` (пакет/зависимости):
```/home/lynx/dev/embedina/Cargo.toml#L1-20
[package]
# TODO(1) fix `authors` and `name` if you didn't use `cargo-generate`
authors = ["asserthq <sanya2003ogorodov@gmail.com>"]
name = "embedina"
edition = "2024"
version = "0.1.0"
```

Зависимости и HAL (фрагмент):
```/home/lynx/dev/embedina/Cargo.toml#L40-70
[dependencies]
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
defmt = "1.0"
defmt-rtt = "1.0"
panic-probe = { version = "1.0", features = ["print-defmt"] }
semihosting = "0.1.20"
# TODO(4) enter your HAL here
# some-hal = "1.2.3"
stm32f3xx-hal = { version = "0.10.0", features = ["ld", "rt", "stm32f303xc"] }
```

Runner и target (в `.cargo/config.toml`):
```/home/lynx/dev/embedina/.cargo/config.toml#L1-20
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
# TODO(2) replace `$CHIP` with your chip's name (see `probe-rs chip list` output)
runner = ["probe-rs", "run", "--chip", "STM32F303VC", "--log-format=oneline"]
...
[build]
target = "thumbv7em-none-eabihf"
```

`src/lib.rs` — общая infra (panic handler, exit, HardFault handler, defmt-test harness):
```/home/lynx/dev/embedina/src/lib.rs#L1-140
#![no_main]
#![no_std]

use defmt_rtt as _; // global logger

// TODO(5) adjust HAL import
// use some_hal as _; // memory layout
use stm32f3xx_hal as _;

use panic_probe as _;

#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

pub fn exit() -> ! {
    semihosting::process::exit(0);
}

#[cortex_m_rt::exception]
unsafe fn HardFault(_frame: &cortex_m_rt::ExceptionFrame) -> ! {
    semihosting::process::exit(1);
}

#[cfg(test)]
#[defmt_test::tests]
mod unit_tests {
    use defmt::assert;

    #[test]
    fn it_works() {
        assert!(true)
    }
}
```

2) Комментарии — что хорошо, что можно улучшить

Хорошо
- У тебя есть корректная основа: `src/lib.rs` реализует общую infra (logger, panic, exit). Это именно то, что должно оставаться в корне и переиспользоваться подпроектами.
- Примеры вынесены в `examples/` — это удобный способ держать демки.
- `.cargo/config.toml` уже содержит удобные `runner`/`target`/`rustflags` — это помогает единообразно прошивать подпроекты.

Что стоит улучшить / структурировать
- `src/adcs/main.rs` сейчас является бинарником, объявленным через `[[bin]]` в корневом `Cargo.toml`. Если `adcs` — большой эксперимент/прототип, лучше вынести его как отдельный подпроект (`projects/adcs`) вместо размещения под `src/`. Тогда `embedina` останется чистой общей библиотекой, а `adcs` — отдельным crate с собственным `Cargo.toml`.
- `edition = "2024"` в `Cargo.toml` может давать проблемы на системах с нестандартным Cargo/rustup (мы уже сталкивались с этим). Я рекомендую:
  - либо понизить `edition` до `2021` (более совместимо), либо
  - положить `rust-toolchain.toml` в корень и фиксировать рабочий toolchain (stable/nightly) для проекта.
- `memory.x` — если понадобятся специфичные linker regions, лучше держать шаблон `memory.x` в корне и/или в каждом подпроекте, который этого требует.
- Подумай о `workspace` (корневой `Cargo.toml` с `[workspace]`), если собираешь множество подпроектов — это упрощает сборку всех crate’ов и CI.

3) Рекомендованная целевая организация (пример)
- Корень `embedina` оставляем как библиотеку + infra + конфиги:
  - `src/lib.rs` — общая infra
  - `.cargo/config.toml` — runner/target
  - `memory.x`, `defmt.x` (если нужно)
  - `examples/` — краткие демки
  - `projects/` — подпроекты (каждый — отдельный crate)
  - `scripts/` — вспомогательные скрипты

Пример структуры:
- embedina/
  - Cargo.toml (workspace)
  - rust-toolchain.toml
  - .cargo/config.toml
  - src/lib.rs
  - examples/*.rs
  - projects/
    - adcs/ (crate)
    - orientation/ (crate)
  - tests/
  - scripts/

Пример `Cargo.toml` workspace (шаблон — вставляй в корень, я показываю пример в /dev/null):
```/dev/null/Cargo.toml#L1-40
[workspace]
members = [
  "projects/adcs",
  "projects/orientation",
  # "."  # можно включить корневой crate, если хочется собирать его тоже
]
```

Пример `projects/adcs/Cargo.toml` (шаблон):
```/dev/null/projects/adcs/Cargo.toml#L1-40
[package]
name = "adcs"
version = "0.1.0"
edition = "2021"

[dependencies]
embedina = { path = "../.." }   # ссылка на общую библиотеку
stm32f3xx-hal = { version = "0.10", features = ["stm32f303xc", "rt"] }
# остальные зависимости (nalgebra, sensors drivers и т.д.)
```

Пример `projects/adcs/src/main.rs` (шаблон):
```/dev/null/projects/adcs/src/main.rs#L1-40
#![no_main]
#![no_std]

use embedina as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::println!("ADCS prototype starting");
    embedina::exit()
}
```

4) Конкретные команды / маленький план миграции (если хочешь — я могу автоматизировать)
- 1) Создать подпроект и перенести `src/adcs/main.rs` туда:
```bash
# из корня репозитория
cargo new --bin projects/adcs --vcs none
# затем скопируй содержимое src/adcs/main.rs -> projects/adcs/src/main.rs
# в projects/adcs/Cargo.toml добавь dependency на embedina: embedina = { path = "../.." }
# в корневом Cargo.toml удалите блок [[bin]] для adcs
```

- 2) Сделать корневой workspace (опционально):
  - Заменить корневой `Cargo.toml` на workspace файл, либо добавить `[workspace]` в отдельный `Cargo.toml` и при необходимости оставить библиотеку в `.`.

- 3) Добавить `rust-toolchain.toml` чтобы фиксировать toolchain (рекомендуется):
```toml
# /home/lynx/dev/embedina/rust-toolchain.toml (пример)
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- 4) Положить шаблон `memory.x` в корень (если нужно) — например:
```/dev/null/memory.x#L1-10
MEMORY {
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  RAM   : ORIGIN = 0x20000000, LENGTH = 40K
}
```
(подставь точные значения для своего MCU)

5) Практические рекомендации для повседневной работы
- Малые быстрые эксперименты: используй `examples/` (команда `cargo run --example <name>`). Это удобно для небольших демо.
- Средние/большие прототипы: делай подпроекты `projects/<name>` (каждый — отдельный crate), подключай `embedina` как path‑dependency.
- Git: используй ветку для рефакторинга `embedina` (если ты меняешь API), подпроекты храни в main. Если прототип в итоге вырастет, можно вынести в отдельный репозиторий.
- CI: настроить workflow, который собирает `--workspace` и проверяет `fmt`/`clippy`.

6) Если хочешь, я могу сейчас
- создать `projects/adcs` и перенести `src/adcs/main.rs` туда (и правки в корневом `Cargo.toml`), или
- сделать `Cargo.toml` workspace-шаблон, или
- сгенерировать `rust-toolchain.toml` и `memory.x` шаблон под твой MCU.

Скажи, какой из пунктов предпочитаешь: «скоро перенести adcs в подпроект», «сделать workspace», «только добавить rust-toolchain и memory.x» — и я внесу изменения/подкажу конкретные команды.
