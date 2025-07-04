pattern = " a1b'k2l`cif/msp\"e3h9o6r^djg>ntq,*5<-u8v.%[$+x!&;:4\\0z7(_?w]#y)="
braille = "⠀⠁⠂⠃⠄⠅⠆⠇⠈⠉⠊⠋⠌⠍⠎⠏⠐⠑⠒⠓⠔⠕⠖⠗⠘⠙⠚⠛⠜⠝⠞⠟⠠⠡⠢⠣⠤⠥⠦⠧⠨⠩⠪⠫⠬⠭⠮⠯⠰⠱⠲⠳⠴⠵⠶⠷⠸⠹⠺⠻⠼⠽⠾⠿"


def main():
    while inp := input().lower():
        output_num = ""
        output_braille = ""
        for i in range(len(inp)):
            if inp[i] in pattern:
                output_num += str(pattern.index(inp[i]))
                output_braille += braille[pattern.index(inp[i])]
            else:
                if inp[i] == "@":
                    output_num += "8"
                    output_braille += braille[8]
                elif inp[i] == "|":
                    output_num += "51"
                    output_braille += braille[51]
                else:
                    raise Exception(f"오류: {inp[i]}")
        print(output_num, output_braille)


if __name__ == "__main__":
    main()
