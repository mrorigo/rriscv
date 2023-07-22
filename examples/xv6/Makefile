XV6_ROOT=../../../../git/xv6-riscv
DEST=$(pwd)

kernel: $(XV6_ROOT)
	(cd $(XV6_ROOT); make clean && make -j 8) && cp $(XV6_ROOT)/kernel/kernel . && cp $(XV6_ROOT)/kernel/kernel.asm kernel.asm
	(cd $(XV6_ROOT); make fs.img ) && cp $(XV6_ROOT)/fs.img ./

$(XV6_ROOT):