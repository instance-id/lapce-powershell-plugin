xdg_data_dir :=  "$HOME/.local/share"

developer := "instanceid"
plugin_name := "lapce-powershell"
plugin_title := developer + "." + plugin_name
plugin_dir := "plugins/"+plugin_title
plugin_lsp := "PowershellEditorServices"
plugin_lspVersion := "latest"

build:
    RUSTFLAGS="-Z wasi-exec-model=reactor" cargo make

build-dev:
    RUSTFLAGS="-Z wasi-exec-model=reactor" cargo make dev

install-nightly: build
    mkdir -p {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/{{plugin_lsp}} || true

install-stable: build
    mkdir -p {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-stable/{{plugin_dir}}/{{plugin_lsp}} || true

install-dev: build-dev
    mkdir -p {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/{{plugin_lsp}} || true

# behold a recipe
fun:
  echo "hi" > tmp.txt
  cat tmp.txt
  rm tmp.txt
