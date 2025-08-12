#!/opt/homebrew/bin/fish

set file $argv[1]
echo "file: $file"

cp $file before

# 797106d622d426d24ad04ece52cc56ca5ac885
# 638ad702b7a6003c
xxd -p $file | tr -d '\n'| grep -ob "797106d622d426d24ad04ece52cc56ca5ac885" | read -d : -l offset _instructions
if test -z "$offset"
    echo "Error: Offset not found"
    exit 1
end
set string '\x01\x45'
set offset (math (math $offset + 8) / 2)
echo "Offset: $offset"
for i in (seq 1 16)
    set string $string'\x01\x00'
end
set string $string'\x82\x80'
for i in (seq 1 231)
    set string $string'\x01\x00'
end
echo "String: $string"
printf $string | dd of=$file seek=$offset bs=1 conv=notrunc
# c.li a0, 0x0
# ret
# nop

cp $file after

espflash flash --monitor --chip esp32c3 $file