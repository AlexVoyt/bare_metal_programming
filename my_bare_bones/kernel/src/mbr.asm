BITS 16                     ; 16 bit code
ORG 0x7C00                  ; BIOS puts us on 0x7c00 linear address, but exact
                            ; combination of segment:offset is undefined

    cli                     ; Disable interrupts
    cld                     ; Clear directional flag for string operations

    in al, 0x92             ; Enable A20 line
	or al, 2
	out 0x92, al

    jmp 0x0:_setup_segments ; Force CS to be zero
_setup_segments:
    xor ax, ax              ; Set all segments except stack segment to zero
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov bx, 0x8000
    mov ss, bx              ; Setup stack
    mov sp, ax

    mov si, msg             ; Load message to print
    call print              ; print message

    mov si, DAPACKET        ; Load our kernel
    mov ah, 0x42
    ; mov dl, 0x80
    int 0x13
    jc short _error_read


    lgdt [GDT_DESCRIPTOR]   ; Load GDT
    mov eax, cr0            ; Enable protected mode
    or al, 1
    mov cr0, eax

    jmp 0x8:_protected_mode_setup_segments

[bits 32]
_protected_mode_setup_segments:
    mov ax, 0x10            ; Set all data segments to appropriate descriptor
    mov ds, ax
    mov es, ax
    mov gs, ax
    mov ss, ax
    mov esp, 0x80000        ; Setup stack
    call 0x7E00             ; Call our kernel entry point
    hlt

[bits 16]
_loop:                      ; Infinite loop
    hlt
    jmp _loop

_error_read:
    mov si, error_msg       ; Load message to print
    call print              ; print message
    jmp _loop

print:
    lodsb                   ; Load character to display
    cmp al, 0
    je print_ret            ; Return if null byte is loaded
    mov ah, 0x0E            ; BIOS interrup number
    mov bx, 0x000F
    int 0x10                ; Call BIOS
    jmp print               ; Continue looping over string

print_ret:
    retn

msg db "My greetings",0
error_msg db "Error while reading kernel from disk!",0
ALIGN 4, db 0                     ; Align Disk Address Packet Structure
DAPACKET:                         ; Disk Address Packet Structure
    db 0x10                       ; Size of packet
    db 0x0                        ; Always zero
    dw SECTORS_TO_READ            ; How many sectors to read - build script passes
                                  ; this value to NASM as define constant
    dw 0x7E00                     ; Destination buffer address (offset)
    dw 0                          ; Destination buffer address (segment)
    dd 1                          ; Lower LBA address
    dd 0                          ; Upper LBA address

ALIGN 8                           ; Align GDT
GDT:                              ; GDT - setup as 32 bit flat memory model
    dq 0x0000000000000000
    dq 0x00CF9A000000FFFF
    dq 0x00CF92000000FFFF
GDT_DESCRIPTOR:
    dw (GDT_DESCRIPTOR - GDT) - 1 ; Limit of GDT
    dd GDT                        ; Base of GDT

times 446 - ($ - $$)  db 0        ; Bootstrap program must be at most 446 bytes long

times 510 - ($ - $$)  db 0        ; Pad rest of MBR with zeroes
dw 0xAA55                         ; MBR signature

INCBIN "build/kernel.flat"        ; Include our kernel
