
C=$1
T=$2

repea() {
	printf "%.0s$1" {1..$2}
}

printf "/"
repea "*" $((C-2))
printf "/\n"

printf "/*"
repea  " " $(( (C-4-$#T)/2 ))
printf "%s" "$T"
repea  " " $(( (C-4-$#T+1)/2 ))
printf "*/\n"

printf "/"
repea  "*" $((C-2))
printf "/\n"
