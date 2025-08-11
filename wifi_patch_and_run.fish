
#!/opt/homebrew/bin/fish

set file $argv[1]
echo "file: $file"

cp $file before

xxd -p $file | tr -d '\n'| grep -ob "638ad702b7a6003c" | read -d : -l offset _instructions
if test -z "$offset"
    echo "Error: Offset not found"
    exit 1
end
set offset (math (math $offset + 8) / 2)
echo "Offset: $offset"
printf '\x01\x45\x82\x80\x13\x00\x00\x00' | dd of=$file seek=$offset bs=1 conv=notrunc
# c.li a0, 0x0
# ret
# nop

cp $file after

espflash flash --monitor --chip esp32c3 $file