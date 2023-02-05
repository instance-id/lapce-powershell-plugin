xdg_data_dir :=  "$HOME/.local/share"

developer := "instanceid"
plugin_name := "lapce-powershell"
plugin_title := developer + "." + plugin_name
plugin_dir := "plugins/"+plugin_title
plugin_lsp := "PowershellEditorServices"
plugin_lspVersion := "latest"

build:
    RUSTFLAGS="-Z wasi-exec-model=reactor" cargo make

install-nightly: build
    mkdir -p {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/{{plugin_lsp}} || true

install-stable: build
    mkdir -p {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/bin
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-nightly/{{plugin_dir}}/{{plugin_lsp}} || true

install-debug: build
    mkdir -p {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}/bin
    yes | cp -i bin/{{plugin_name}}.wasm {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}/bin
    yes | cp -i volt.toml {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}/
#    rm -rd {{xdg_data_dir}}/lapce-debug/{{plugin_dir}}/{{plugin_lsp}} || true

# behold a recipe
fun:
  echo "hi" > tmp.txt
  cat tmp.txt
  rm tmp.txt
