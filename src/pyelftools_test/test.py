from elftools.elf.elffile import ELFFile

syms=[]

def find_symbol_address(elf_file, symbol_name):
    elffile = ELFFile(elf_file)
    # シンボルテーブルを探す
    for section in elffile.iter_sections():
        if not hasattr(section, 'get_symbol'):
            continue
        # シンボルテーブル内のシンボルをループ
        for sym in section.iter_symbols():
            syms.append(sym)
            if symbol_name in sym.name:
                print(f"{sym.name} : {hex(sym['st_value'])}")  # シンボルのアドレスを返す

if __name__ == '__main__':
    print("elf valiavle address searcher")  
    # 使用例
    elf_path = 'C://Users//takaa//Dropbox//project//STM32Cube//VScode_workspace//G431_LEDBlink//build//debug//build//G431_LEDBlink.elf'
    symbol_name = 'cnt'

    with open(elf_path, "rb") as f:
        address = find_symbol_address(f, symbol_name)