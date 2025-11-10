# How do we made it

DONT RUN IT ON VSCODE, RUN IT DIRECTLY ON A TERMINAL

## Useful cmds
- `rustup component list`
- `rustup show`


## Explanation of how it load 

- Démarrage du système:

    - Lorsqu'un ordinateur est allumé, le firmware (BIOS ou UEFI) est exécuté. Ce code est stocké dans la ROM de la carte mère.
    - Le firmware effectue un auto-test au démarrage, détecte la RAM disponible et initialise le CPU et le matériel.

- Recherche d'un disque amorçable:

    - Après l'initialisation, le firmware cherche un disque amorçable (comme un disque dur, une clé USB, etc.).
    - Si un disque amorçable est trouvé, le contrôle est transféré au bootloader, qui est une portion de code exécutable de 512 octets au début du disque.

- Bootloader: 

    - Le bootloader détermine l'emplacement de l'image du kernel sur le disque et la charge en mémoire
    - Le bootloader doit également changer le mode du CPU de mode réel 16 bits à mode protégé 32 bits, puis à mode long 64 bits. Cela permet d'accéder aux registres 64 bits et à l'ensemble de la mémoire principale.

- Multiboot : 

    - Ce standard définit une interface entre le bootloader et le système d'exploitation, permettant à n'importe quel bootloader conforme au Multiboot de charger n'importe quel système d'exploitation conforme.
    - Pour rendre un kernel conforme au Multiboot, il suffit d'insérer un header Multiboot au début du fichier du kernel. Cela simplifie le démarrage d'un système d'exploitation à partir de GRUB (GNU Grand Unified Bootloader), qui est le bootloader le plus populaire pour les systèmes Linux.

## Explanation of the Makefile

Compilation of OS in Rust, with an x86 arch, a linker and iso image bootable with GRUB. 

- `-Z build-std=core,alloc` : building with specific librairies, cuz we dont have the entire librairie from Rust (std) only core and alloc, which is common for bare-metal OS.

- `--target=i386.json` : Our target is an architecture i386 (32bits) : defined model of the CPU, every informations linked to x86 arch.

- `--release`: optimisation activated.

[...]

## BOOTLOADER

1- boot.asm : bootloader file (follow comments on the file)

2- main.rs : routine with main.rs
    - printing
3- linker.ld : link routine (link object files into the final kernel) with boot.asm.
    - it defines memory layout for your kernel and establishes how different sections of the asm code will be organized in memory.
4- grub.cfg: indiquer a grub le binaire, notre image

### GDTR

- Creation de 7 entrees : null, kernel/user, code/data/stack
- Creation d'un octet d'acces 0x96/0xF6 pour que le CPU detecte les stackoverflows
- Copy la GDT a l'adresse 0x00000800, construit le pseudo-descriptor (limit/base) et prepare un pointeur vers le kernel_main
- Charge le pointeur, recharge les segments, bascule StackSegment+ESP sur la stack kernel defini en ASM puis `ljmp` pour recharger CodeSegment sur le selecteur kernel code.

### VGA buffer

1- Representing color using c-Like Enum, add TextBuffer as struct where we define the buffer height and width. Volatile dependencies tells the compiler that the write has side effects and should not be optimized away (cuz Rust compiler always try to optimize).

2- en francais pcq un peu complique. Ensuite, creation d'une struct Writer qui sera la structure fonctionnelle d'ecriture sur le buffer. La macro lazy static permet d'init des variables statique, et elles sont statiques seulement lorsqu'elles y accedent pour la premiere fois. 
`static ref` est une ref statique immuable mais le contenu peut etre mutable grace au Mutex. Mutex<Writer> est une structure de synchronisation qui assure que seul un thread peut acceder a la ressource partagee. Ca garantit l'acces sur au Writer dans un env multitache ou multithread. 

3- println function using macro_rules!

4- Panic functions