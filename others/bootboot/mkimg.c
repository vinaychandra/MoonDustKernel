/*
 * images/mkimg.c
 *
 * Copyright (C) 2017 - 2020 bzt (bztsrc@gitlab)
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
 * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 *
 * This file is part of the BOOTBOOT Protocol package.
 * @brief Small tool to create FAT, GPT disk or CDROM images with BOOTBOOT
 *
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <time.h>
#define __USE_MISC 1            /* due to DT_* enums */
#include <dirent.h>

long int read_size;
time_t t;
struct tm *ts;
int nextcluster = 3, bpc;
unsigned char *fs, *data;
uint16_t *fat16_1 = NULL, *fat16_2;
uint32_t *fat32_1 = NULL, *fat32_2;

/**
 * Read file into memory
 */
unsigned char* readfileall(char *file)
{
    unsigned char *data=NULL;
    FILE *f;
    read_size=0;
    if(!file || !*file) return NULL;
    f=fopen(file,"r");
    if(f){
        fseek(f,0L,SEEK_END);
        read_size=ftell(f);
        fseek(f,0L,SEEK_SET);
        data=(unsigned char*)malloc(read_size+1);
        if(data==NULL) { fprintf(stderr,"mkimg: Unable to allocate %ld memory\n",read_size+1); exit(1); }
        memset(data,0,read_size+1);
        fread(data,read_size,1,f);
        fclose(f);
    }
    return data;
}

/**
 * Set integers in byte arrays
 */
int getint(unsigned char *ptr) { return (unsigned char)ptr[0]+(unsigned char)ptr[1]*256+(unsigned char)ptr[2]*256*256+ptr[3]*256*256*256; }
void setint(int val, unsigned char *ptr) { memcpy(ptr,&val,4); }
void setinte(int val, unsigned char *ptr) { char *v=(char*)&val; memcpy(ptr,&val,4); ptr[4]=v[3]; ptr[5]=v[2]; ptr[6]=v[1]; ptr[7]=v[0]; }

/**
 * CRC-stuff
 */
unsigned int crc32_lookup[256]={
    0x00000000,0x77073096,0xee0e612c,0x990951ba,0x076dc419,0x706af48f,0xe963a535,0x9e6495a3,0x0edb8832,
    0x79dcb8a4,0xe0d5e91e,0x97d2d988,0x09b64c2b,0x7eb17cbd,0xe7b82d07,0x90bf1d91,0x1db71064,0x6ab020f2,
    0xf3b97148,0x84be41de,0x1adad47d,0x6ddde4eb,0xf4d4b551,0x83d385c7,0x136c9856,0x646ba8c0,0xfd62f97a,
    0x8a65c9ec,0x14015c4f,0x63066cd9,0xfa0f3d63,0x8d080df5,0x3b6e20c8,0x4c69105e,0xd56041e4,0xa2677172,
    0x3c03e4d1,0x4b04d447,0xd20d85fd,0xa50ab56b,0x35b5a8fa,0x42b2986c,0xdbbbc9d6,0xacbcf940,0x32d86ce3,
    0x45df5c75,0xdcd60dcf,0xabd13d59,0x26d930ac,0x51de003a,0xc8d75180,0xbfd06116,0x21b4f4b5,0x56b3c423,
    0xcfba9599,0xb8bda50f,0x2802b89e,0x5f058808,0xc60cd9b2,0xb10be924,0x2f6f7c87,0x58684c11,0xc1611dab,
    0xb6662d3d,0x76dc4190,0x01db7106,0x98d220bc,0xefd5102a,0x71b18589,0x06b6b51f,0x9fbfe4a5,0xe8b8d433,
    0x7807c9a2,0x0f00f934,0x9609a88e,0xe10e9818,0x7f6a0dbb,0x086d3d2d,0x91646c97,0xe6635c01,0x6b6b51f4,
    0x1c6c6162,0x856530d8,0xf262004e,0x6c0695ed,0x1b01a57b,0x8208f4c1,0xf50fc457,0x65b0d9c6,0x12b7e950,
    0x8bbeb8ea,0xfcb9887c,0x62dd1ddf,0x15da2d49,0x8cd37cf3,0xfbd44c65,0x4db26158,0x3ab551ce,0xa3bc0074,
    0xd4bb30e2,0x4adfa541,0x3dd895d7,0xa4d1c46d,0xd3d6f4fb,0x4369e96a,0x346ed9fc,0xad678846,0xda60b8d0,
    0x44042d73,0x33031de5,0xaa0a4c5f,0xdd0d7cc9,0x5005713c,0x270241aa,0xbe0b1010,0xc90c2086,0x5768b525,
    0x206f85b3,0xb966d409,0xce61e49f,0x5edef90e,0x29d9c998,0xb0d09822,0xc7d7a8b4,0x59b33d17,0x2eb40d81,
    0xb7bd5c3b,0xc0ba6cad,0xedb88320,0x9abfb3b6,0x03b6e20c,0x74b1d29a,0xead54739,0x9dd277af,0x04db2615,
    0x73dc1683,0xe3630b12,0x94643b84,0x0d6d6a3e,0x7a6a5aa8,0xe40ecf0b,0x9309ff9d,0x0a00ae27,0x7d079eb1,
    0xf00f9344,0x8708a3d2,0x1e01f268,0x6906c2fe,0xf762575d,0x806567cb,0x196c3671,0x6e6b06e7,0xfed41b76,
    0x89d32be0,0x10da7a5a,0x67dd4acc,0xf9b9df6f,0x8ebeeff9,0x17b7be43,0x60b08ed5,0xd6d6a3e8,0xa1d1937e,
    0x38d8c2c4,0x4fdff252,0xd1bb67f1,0xa6bc5767,0x3fb506dd,0x48b2364b,0xd80d2bda,0xaf0a1b4c,0x36034af6,
    0x41047a60,0xdf60efc3,0xa867df55,0x316e8eef,0x4669be79,0xcb61b38c,0xbc66831a,0x256fd2a0,0x5268e236,
    0xcc0c7795,0xbb0b4703,0x220216b9,0x5505262f,0xc5ba3bbe,0xb2bd0b28,0x2bb45a92,0x5cb36a04,0xc2d7ffa7,
    0xb5d0cf31,0x2cd99e8b,0x5bdeae1d,0x9b64c2b0,0xec63f226,0x756aa39c,0x026d930a,0x9c0906a9,0xeb0e363f,
    0x72076785,0x05005713,0x95bf4a82,0xe2b87a14,0x7bb12bae,0x0cb61b38,0x92d28e9b,0xe5d5be0d,0x7cdcefb7,
    0x0bdbdf21,0x86d3d2d4,0xf1d4e242,0x68ddb3f8,0x1fda836e,0x81be16cd,0xf6b9265b,0x6fb077e1,0x18b74777,
    0x88085ae6,0xff0f6a70,0x66063bca,0x11010b5c,0x8f659eff,0xf862ae69,0x616bffd3,0x166ccf45,0xa00ae278,
    0xd70dd2ee,0x4e048354,0x3903b3c2,0xa7672661,0xd06016f7,0x4969474d,0x3e6e77db,0xaed16a4a,0xd9d65adc,
    0x40df0b66,0x37d83bf0,0xa9bcae53,0xdebb9ec5,0x47b2cf7f,0x30b5ffe9,0xbdbdf21c,0xcabac28a,0x53b39330,
    0x24b4a3a6,0xbad03605,0xcdd70693,0x54de5729,0x23d967bf,0xb3667a2e,0xc4614ab8,0x5d681b02,0x2a6f2b94,
    0xb40bbe37,0xc30c8ea1,0x5a05df1b,0x2d02ef8d};

unsigned int crc32a_calc(char *start, int length)
{
    unsigned int crc32_val=0xffffffff;
    while(length--) crc32_val=(crc32_val>>8)^crc32_lookup[(crc32_val&0xff)^(unsigned char)*start++];
    crc32_val^=0xffffffff;
    return crc32_val;
}

/**
 * Create a ROM image of the initrd
 */
void initrdrom()
{
    int i, size;
    unsigned char *fs, *buf, c=0;
    FILE *f;

    fs=readfileall("initrd.bin"); size=((read_size+32+511)/512)*512;
    if(fs==NULL) { fprintf(stderr,"mkimg: unable to load initrd.bin\n"); exit(2); }
    buf=(unsigned char*)malloc(size+1);
    /* Option ROM header */
    buf[0]=0x55; buf[1]=0xAA; buf[2]=(read_size+32+511)/512;
    /* asm "xor ax,ax; retf" */
    buf[3]=0x31; buf[4]=0xC0; buf[5]=0xCB;
    /* identifier, size and data */
    memcpy(buf+8,"INITRD",6);
    memcpy(buf+16,&read_size,4);
    memcpy(buf+32,fs,read_size);
    /* checksum */
    for(i=0;i<size;i++) c+=buf[i];
    buf[6]=(unsigned char)((int)(256-c));
    /* write out */
    f=fopen("initrd.rom","wb");
    if(!f) { fprintf(stderr,"mkimg: unable to write initrd.rom\n"); exit(3); }
    fwrite(buf,size,1,f);
    fclose(f);
}

/**
 * Add a FAT directory entry
 */
unsigned char *adddirent(unsigned char *ptr, char *name, int type, int cluster, int size)
{
    int i, j;
    memset(ptr, ' ', 11);
    if(name[0] == '.') strcpy((char*)ptr, name);
    else
        for(i = j = 0; j < 11 && name[i]; i++, j++) {
            if(name[i] >= 'a' && name[i] <= 'z') ptr[j] = name[i] - ('a' - 'A');
            else if(name[i] == '.') { j = 7; continue; }
            else ptr[j] = name[i];
        }
    ptr[0xB] = type;
    i = (ts->tm_hour << 11) | (ts->tm_min << 5) | (ts->tm_sec/2);
    ptr[0xE] = ptr[0x16] = i & 0xFF; ptr[0xF] = ptr[0x17] = (i >> 8) & 0xFF;
    i = ((ts->tm_year+1900-1980) << 9) | ((ts->tm_mon+1) << 5) | (ts->tm_mday);
    ptr[0x10] = ptr[0x12] = ptr[0x18] = i & 0xFF; ptr[0x11] = ptr[0x13] = ptr[0x19] = (i >> 8) & 0xFF;
    ptr[0x1A] = cluster & 0xFF; ptr[0x1B] = (cluster >> 8) & 0xFF;
    ptr[0x14] = (cluster >> 16) & 0xFF; ptr[0x15] = (cluster >> 24) & 0xFF;
    ptr[0x1C] = size & 0xFF; ptr[0x1D] = (size >> 8) & 0xFF;
    ptr[0x1E] = (size >> 16) & 0xFF; ptr[0x1F] = (size >> 24) & 0xFF;
    return ptr + 32;
}

/**
 * Recursively parse the boot directory and add entries to the boot partition image
 */
void parsedir(unsigned char *ptr, char *directory, int parent)
{
    DIR *dir;
    struct dirent *ent;
    char full[1024];
    unsigned char *tmp, *ptr2;
    int i;

    if ((dir = opendir(directory)) != NULL) {
        while ((ent = readdir(dir)) != NULL) {
            if(ent->d_name[0] == '.') continue;
            sprintf(full,"%s/%s",directory,ent->d_name);
            ptr2 = data + nextcluster * bpc;
            if(ent->d_type == DT_DIR) {
                ptr = adddirent(ptr, ent->d_name, 0x10, nextcluster, 0);
                if(fat16_1) fat16_1[nextcluster] = fat16_2[nextcluster] = 0xFFFF;
                else fat32_1[nextcluster] = fat32_2[nextcluster] = 0x0FFFFFFF;
                ptr2 = adddirent(ptr2, ".", 0x10, nextcluster, 0);
                ptr2 = adddirent(ptr2, "..", 0x10, parent, 0);
                nextcluster++;
                parsedir(ptr2, full, nextcluster - 1);
            } else
            if(ent->d_type == DT_REG) {
                tmp = readfileall(full);
                if(tmp) {
                    ptr = adddirent(ptr, ent->d_name, 0, nextcluster, read_size);
                    /* make sure LOADER is 2048 bytes aligned */
                    if(tmp[0]==0x55 && tmp[1]==0xAA && tmp[3]==0xE9 && tmp[8]=='B' && tmp[12]=='B' && (int)(ptr2-fs) & 2047) {
                        i = 2048 - ((int)(ptr2-fs) & 2047); ptr2 += i;
                        nextcluster += i / bpc;
                    }
                    memcpy(ptr2, tmp, read_size);
                    for(i = 0; i < (int)read_size; i += bpc, nextcluster++) {
                        if(fat16_1) fat16_1[nextcluster] = fat16_2[nextcluster] = nextcluster+1;
                        else fat32_1[nextcluster] = fat32_2[nextcluster] = nextcluster+1;
                    }
                    if(fat16_1) fat16_1[nextcluster-1] = fat16_2[nextcluster-1] = 0xFFFF;
                    else fat32_1[nextcluster-1] = fat32_2[nextcluster-1] = 0x0FFFFFFF;
                }
            }
        }
    }
}

/**
 * Create bootpart.bin with FAT16 or FAT32
 */
int createfat(int fattype, int partsize, char *directory)
{
    unsigned char *rootdir;
    int i, spf;
    FILE *f;

    if(fattype != 16 && fattype != 32) { fprintf(stderr,"mkimg: unsupported FAT type. Use 16 or 32.\n"); exit(1); }
    if(fattype == 16 && partsize < 16*1024*1024) partsize = 16*1024*1024;
    if(fattype == 16 && partsize >= 32*1024*1024) fattype = 32;
    if(fattype == 32 && partsize < 33*1024*1024) partsize = 33*1024*1024;

    fs = malloc(partsize);
    if(fs==NULL) { fprintf(stderr,"mkimg: unable to allocate %d bytes\n", partsize); exit(2); }
    memset(fs, 0, partsize);
    /* Volume Boot Record */
    fs[0] = 0xEB; fs[1] = fattype == 16 ? 0x3C : 0x58; fs[2] = 0x90;
    memcpy(fs + 3, "MSWIN4.1 ", 8); fs[0xC] = 2; fs[0xD] = 4; fs[0x10] = 2; fs[0x15] = 0xF8; fs[0x1FE] = 0x55; fs[0x1FF] = 0xAA;
    fs[0x18] = 0x20; fs[0x1A] = 0x40;
    i = (partsize + 511) / 512;
    if(fattype == 16) {
        fs[0xD] = 4; fs[0xE] = 4; fs[0x12] = 2; fs[0x13] = i & 0xFF; fs[0x14] = (i >> 8) & 0xFF;
        bpc = fs[0xD] * 512;
        spf = ((partsize/bpc)*2 + 511) / 512;
        fs[0x16] = spf & 0xFF; fs[0x17] = (spf >> 8) & 0xFF;
        fs[0x24] = 0x80; fs[0x26] = 0x29; fs[0x27] = 0xB0; fs[0x28] = 0x07; fs[0x29] = 0xB0; fs[0x2A] = 0x07;
        memcpy(fs + 0x2B, "EFI System FAT16   ", 19);
        rootdir = fs + (spf*fs[0x10]+fs[0xE]) * 512;
        data = rootdir + ((((fs[0x12]<<8)|fs[0x11])*32 - 4096) & ~2047);
        fat16_1 = (uint16_t*)(&fs[fs[0xE] * 512]);
        fat16_2 = (uint16_t*)(&fs[(fs[0xE]+spf) * 512]);
        fat16_1[0] = fat16_2[0] = 0xFFF8; fat16_1[1] = fat16_2[1] = 0xFFFF;
    } else {
        fs[0xD] = 1; fs[0xE] = 0x20;
        fs[0x20] = i & 0xFF; fs[0x21] = (i >> 8) & 0xFF; fs[0x22] = (i >> 16) & 0xFF; fs[0x23] = (i >> 24) & 0xFF;
        bpc = fs[0xD] * 512;
        spf = ((partsize/bpc)*4) / 512 - 8;
        fs[0x24] = spf & 0xFF; fs[0x25] = (spf >> 8) & 0xFF; fs[0x26] = (spf >> 16) & 0xFF; fs[0x27] = (spf >> 24) & 0xFF;
        fs[0x2C] = 2; fs[0x30] = 1; fs[0x32] = 6; fs[0x40] = 0x80;
        fs[0x42] = 0x29; fs[0x43] = 0xB0; fs[0x44] = 0x07; fs[0x45] = 0xB0; fs[0x46] = 0x07;
        memcpy(fs + 0x47, "EFI System FAT32   ", 19);
        memcpy(fs + 0x200, "RRaA", 4); memcpy(fs + 0x3E4, "rrAa", 4);
        for(i = 0; i < 8; i++) fs[0x3E8 + i] = 0xFF;
        fs[0x3FE] = 0x55; fs[0x3FF] = 0xAA;
        memcpy(fs + 0xC00, fs, 512);
        rootdir = fs + (spf*fs[0x10]+fs[0xE]) * 512;
        data = rootdir - 1024;
        fat32_1 = (uint32_t*)(&fs[fs[0xE] * 512]);
        fat32_2 = (uint32_t*)(&fs[(fs[0xE]+spf) * 512]);
        fat32_1[0] = fat32_2[0] = fat32_1[2] = fat32_2[2] = 0x0FFFFFF8; fat32_1[1] = fat32_2[1] = 0x0FFFFFFF;
    }
    /* label in root directory */
    rootdir = adddirent(rootdir, ".", 8, 0, 0);
    memcpy(rootdir - 32, "EFI System ", 11);
    /* add contents of the boot directory to the image */
    parsedir(rootdir, directory, 0);
    /* update fields in FS Information Sector */
    if(fattype == 32) {
        nextcluster -= 2;
        i = ((partsize - (spf*fs[0x10]+fs[0xE]) * 512)/bpc) - nextcluster;
        fs[0x3E8] = i & 0xFF; fs[0x3E9] = (i >> 8) & 0xFF;
        fs[0x3EA] = (i >> 16) & 0xFF; fs[0x3EB] = (i >> 24) & 0xFF;
        fs[0x3EC] = nextcluster & 0xFF; fs[0x3ED] = (nextcluster >> 8) & 0xFF;
        fs[0x3EE] = (nextcluster >> 16) & 0xFF; fs[0x3EF] = (nextcluster >> 24) & 0xFF;

    }
    /* write out */
    f=fopen("bootpart.bin","wb");
    if(!f) { fprintf(stderr,"mkimg: unable to write bootpart.bin\n"); exit(3); }
    fwrite(fs,partsize,1,f);
    fclose(f);
    free(fs);
    return 0;
}

/**
 * Create a hybrid disk image from partition image with initrd in it
 */
int createdisk(int disksize, char *diskname)
{
    unsigned long int i,j=0,gs=63*512,es,bbs=0;
    unsigned long int uuid[4]={0x12345678,0x12345678,0x12345678,0x12345678};
    unsigned char *esp, *gpt, *iso, *p, *loader, *loader2;
    char isodate[17];
    FILE *f;

    esp=readfileall("bootpart.bin");   es=read_size;
    gpt=malloc(gs+512);
    memset(gpt,0,gs+512);
    iso=malloc(32768);
    memset(iso,0,32768);
    if(disksize<64*1024*1024) disksize = 64*1024*1024;
    /* make the UUID unique */
    uuid[1] ^= (unsigned long int)t;

    /* MBR / VBR stage 1 loader */
    loader=readfileall("../others/bootboot/boot.bin");
    if(loader==NULL) {
        fprintf(stderr,"mkimg: stage1 ../others/bootboot/boot.bin not found, creating non-bootable disk\n");
        loader=malloc(512);
        memset(loader,0,512);
    } else {
        memset(loader+0x1B8,0,0x1FE - 0x1B8);
    }
    /* search for stage2 loader (FS0:\BOOTBOOT\LOADER) */
    if(es>0) {
        for(i=0;i<es-512;i+=512) {
            if((unsigned char)esp[i+0]==0x55 &&
               (unsigned char)esp[i+1]==0xAA &&
               (unsigned char)esp[i+3]==0xE9 &&
               (unsigned char)esp[i+8]=='B' &&
               (unsigned char)esp[i+12]=='B') {
                bbs=((i+65536)/512);
                break;
            }
        }
    }
    /* failsafe */
    if(!bbs) {
        fprintf(stderr,"mkimg: FS0:\\BOOTBOOT\\LOADER not found, adding stage2 before the boot partition\n");
        loader2 = readfileall("../bootboot.bin");
        if(!loader2) fprintf(stderr,"mkimg: stage2 ../bootboot.bin not found, creating non-bootable disk\n");
        else { memcpy(gpt + 16384, loader2, read_size); bbs = 16384 / 512; }
    }
    /* save stage2 address into stage1 */
    setint(bbs,loader+0x1B0);
    /* WinNT disk id */
    setint(uuid[0],loader+0x1B8);
    /* magic */
    loader[0x1FE]=0x55; loader[0x1FF]=0xAA;

    /* copy stage1 loader into VBR too */
    if(loader[0]!=0 && es>0) {
        /* skip BPB, but copy jump and OEM */
        memcpy(esp, loader, 11);
        memcpy(esp + 0x5A, loader + 0x5A, 0x1B8 - 0x5A);
        esp[0x1FE]=0x55; esp[0x1FF]=0xAA;
    }

    /* generate PMBR partitioning table */
    j=0x1C0;
    if(es>0) {
        /* MBR, EFI System Partition / boot partition. Don't use 0xEF as type, RPi doesn't like that */
        loader[j-2]=0x80;                           /* bootable flag */
        setint(129,loader+j);                       /* start CHS */
        loader[j+2]=esp[0x39]=='1' ? 0xE : 0xC;     /* type, LBA FAT16 (0xE) or FAT32 (0xC) */
        setint(((gs+es)/512)+2,loader+j+4);         /* end CHS */
        setint(128,loader+j+6);                     /* start LBA */
        setint(((es)/512),loader+j+10);             /* number of sectors */
        j+=16;
    }
    /* MBR, protective GPT entry */
    setint(1,loader+j);                             /* start CHS */
    loader[j+2]=0xEE;                               /* type */
    setint((gs/512)+1,loader+j+4);                  /* end CHS */
    setint(1,loader+j+6);                           /* start LBA */
    setint((gs/512),loader+j+10);                   /* number of sectors */
    j+=16;

    /* GPT header */
    memset(gpt,0,gs);
    memcpy(gpt,"EFI PART",8);                       /* magic */
    setint(1,gpt+10);                               /* revision */
    setint(92,gpt+12);                              /* size */
    setint(1,gpt+24);                               /* primary LBA */
    setint(disksize/512-1,gpt+32);                  /* secondary LBA */
    setint((gs/512)+1,gpt+40);                      /* first usable LBA */
    setint((disksize/512)-1,gpt+48);                /* last usable LBA */
    setint(uuid[0],gpt+56);                         /* disk UUID */
    setint(uuid[1],gpt+60);
    setint(uuid[2],gpt+64);
    setint(uuid[3],gpt+68);
    setint(2,gpt+72);                               /* partitioning table LBA */
    setint(es?1:0,gpt+80);                          /* number of entries */
    setint(128,gpt+84);                             /* size of one entry */

    p=gpt+512;
    /* GPT, EFI System Partition (ESP, /boot) */
    if(es>0) {
        setint(0x0C12A7328,p);                      /* entry type */
        setint(0x011D2F81F,p+4);
        setint(0x0A0004BBA,p+8);
        setint(0x03BC93EC9,p+12);
        setint(uuid[0]+1,p+16);                     /* partition UUID */
        setint(uuid[1],p+20);
        setint(uuid[2],p+24);
        setint(uuid[3],p+28);
        setint(128,p+32);                           /* start LBA */
        setint(((es)/512)+127,p+40);                /* end LBA */
        memcpy(p+64,L"EFI System Partition",42);    /* name */
        p+=128;
    }

    /* calculate checksums */
    /* partitioning table */
    i=(int)(gpt[80]*gpt[84]);
    setint(crc32a_calc((char*)gpt+512,i),gpt+88);
    /* header */
    i=getint(gpt+12);   /* size of header */
    setint(0,gpt+16);   /* calculate as zero */
    setint(crc32a_calc((char*)gpt,i),gpt+16);

    /* ISO9660 cdrom image part */
    if(bbs%4!=0) {
        fprintf(stderr,"mkimg: %s (LBA %ld, offs %lx)\n","Stage2 is not 2048 byte sector aligned", bbs, bbs*512);
        exit(3);
    }
    sprintf((char*)&isodate, "%04d%02d%02d%02d%02d%02d00",
        ts->tm_year+1900,ts->tm_mon+1,ts->tm_mday,ts->tm_hour,ts->tm_min,ts->tm_sec);
    /* 16th sector: Primary Volume Descriptor */
    iso[0]=1;   /* Header ID */
    memcpy(&iso[1], "CD001", 5);
    iso[6]=1;   /* version */
    for(i=8;i<72;i++) iso[i]=' ';
    memcpy(&iso[40], "BOOTBOOT_CD", 11);   /* Volume Identifier */
    setinte((65536+es+2047)/2048, &iso[80]);
    iso[120]=iso[123]=1;        /* Volume Set Size */
    iso[124]=iso[127]=1;        /* Volume Sequence Number */
    iso[129]=iso[130]=8;        /* logical blocksize (0x800) */
    iso[156]=0x22;              /* root directory recordsize */
    setinte(20, &iso[158]);     /* root directory LBA */
    setinte(2048, &iso[166]);   /* root directory size */
    iso[174]=ts->tm_year;       /* root directory create date */
    iso[175]=ts->tm_mon+1;
    iso[176]=ts->tm_mday;
    iso[177]=ts->tm_hour;
    iso[178]=ts->tm_min;
    iso[179]=ts->tm_sec;
    iso[180]=0;                 /* timezone UTC (GMT) */
    iso[181]=2;                 /* root directory flags (0=hidden,1=directory) */
    iso[184]=1;                 /* root directory number */
    iso[188]=1;                 /* root directory filename length */
    for(i=190;i<813;i++) iso[i]=' ';    /* Volume data */
    memcpy(&iso[318], "BOOTBOOT <HTTPS://GITLAB.COM/BZTSRC/BOOTBOOT>", 45);
    memcpy(&iso[446], "BOOTBOOT MKIMG", 14);
    memcpy(&iso[574], "BOOTBOOT CD", 11);
    for(i=702;i<813;i++) iso[i]=' ';    /* file descriptors */
    memcpy(&iso[813], &isodate, 16);    /* volume create date */
    memcpy(&iso[830], &isodate, 16);    /* volume modify date */
    for(i=847;i<863;i++) iso[i]='0';    /* volume expiration date */
    for(i=864;i<880;i++) iso[i]='0';    /* volume shown date */
    iso[881]=1;                         /* filestructure version */
    for(i=883;i<1395;i++) iso[i]=' ';   /* file descriptors */
    /* 17th sector: Boot Record Descriptor */
    iso[2048]=0;    /* Header ID */
    memcpy(&iso[2049], "CD001", 5);
    iso[2054]=1;    /* version */
    memcpy(&iso[2055], "EL TORITO SPECIFICATION", 23);
    setinte(19, &iso[2048+71]);         /* Boot Catalog LBA */
    /* 18th sector: Volume Descritor Terminator */
    iso[4096]=0xFF; /* Header ID */
    memcpy(&iso[4097], "CD001", 5);
    iso[4102]=1;    /* version */
    /* 19th sector: Boot Catalog */
    /* --- BIOS, Validation Entry + Initial/Default Entry --- */
    iso[6144]=1;    /* Header ID, Validation Entry */
    iso[6145]=0;    /* Platform 80x86 */
    iso[6172]=0xaa; /* magic bytes */
    iso[6173]=0x55;
    iso[6174]=0x55;
    iso[6175]=0xaa;
    iso[6176]=0x88; /* Bootable, Initial/Default Entry */
    iso[6182]=4;    /* Sector Count */
    setint(128/4, &iso[6184]);  /* Boot Record LBA */
    /* --- UEFI, Final Section Header Entry + Section Entry --- */
    iso[6208]=0x91; /* Header ID, Final Section Header Entry */
    iso[6209]=0xEF; /* Platform EFI */
    iso[6210]=1;    /* Number of entries */
    iso[6240]=0x88; /* Bootable, Section Entry */
    setint(128/4, &iso[6248]);  /* ESP Start LBA */
    /* 20th sector: Root Directory */
    /* . */
    iso[8192]=0x22;              /* recordsize */
    setinte(20, &iso[8194]);     /* LBA */
    setinte(2048, &iso[8202]);   /* size */
    iso[8210]=ts->tm_year;       /* date */
    iso[8211]=ts->tm_mon+1;
    iso[8212]=ts->tm_mday;
    iso[8213]=ts->tm_hour;
    iso[8214]=ts->tm_min;
    iso[8215]=ts->tm_sec;
    iso[8216]=0;                 /* timezone UTC (GMT) */
    iso[8217]=2;                 /* flags (0=hidden,1=directory) */
    iso[8220]=1;                 /* serial */
    iso[8224]=1;                 /* filename length */
    /* .. */
    iso[8226]=0x22;              /* recordsize */
    setinte(20, &iso[8228]);     /* LBA */
    setinte(2048, &iso[8236]);   /* size */
    iso[8244]=ts->tm_year;       /* date */
    iso[8245]=ts->tm_mon+1;
    iso[8246]=ts->tm_mday;
    iso[8247]=ts->tm_hour;
    iso[8248]=ts->tm_min;
    iso[8249]=ts->tm_sec;
    iso[8250]=0;                 /* timezone UTC (GMT) */
    iso[8251]=2;                 /* flags (0=hidden,1=directory) */
    iso[8254]=1;                 /* serial */
    iso[8258]=2;                 /* filename length */
    /* README.TXT */
    iso[8260]=0x22+12;           /* recordsize */
    setinte(21, &iso[8262]);     /* LBA */
    setinte(130, &iso[8270]);    /* size */
    iso[8278]=ts->tm_year;       /* date */
    iso[8279]=ts->tm_mon+1;
    iso[8280]=ts->tm_mday;
    iso[8281]=ts->tm_hour;
    iso[8282]=ts->tm_min;
    iso[8283]=ts->tm_sec;
    iso[8284]=0;                 /* timezone UTC (GMT) */
    iso[8285]=0;                 /* flags (0=hidden,1=directory) */
    iso[8288]=1;                 /* serial */
    iso[8292]=12;                /* filename length */
    memcpy(&iso[8293], "README.TXT;1", 12);
    /* 21th sector: contents of README.TXT */
    memcpy(&iso[10240], "BOOTBOOT Live Image\r\n\r\nBootable as\r\n"
        " - CDROM (El Torito, UEFI)\r\n"
        " - USB stick (BIOS, Multiboot, UEFI)\r\n"
        " - SD card (Raspberry Pi 3+)", 130);

    f=fopen(diskname,"wb");
    if(!f) {
        fprintf(stderr,"mkimg: unable to write %s\n",diskname);
        exit(2);
    }
    /* (P)MBR */
    fwrite(loader,512,1,f);
    /* GPT header + entries */
    fwrite(gpt,gs,1,f);
    /* ISO9660 descriptors */
    fwrite(iso,32768,1,f);
    /* Partitions */
    if(es>0)
        fwrite(esp,es,1,f);
    fseek(f,disksize-gs,SEEK_SET);

    /* GPT entries again */
    fwrite(gpt+512,gs-512,1,f);
    /* GPT secondary header */
    i=getint(gpt+32);
    setint(getint(gpt+24),gpt+32);                     /* secondary lba */
    setint(i,gpt+24);                                  /* primary lba */

    setint((i*512-gs)/512+1,gpt+72);                   /* partition lba */
    i=getint(gpt+12);   /* size of header */
    setint(0,gpt+16);   /* calculate with zero */
    setint(crc32a_calc((char*)gpt,i),gpt+16);
    fwrite(gpt,512,1,f);
    fclose(f);
    return 1;
}

/*** ELF64 defines and structs ***/
#define ELFMAG      "\177ELF"
#define SELFMAG     4
#define EI_CLASS    4       /* File class byte index */
#define ELFCLASS64  2       /* 64-bit objects */
#define EI_DATA     5       /* Data encoding byte index */
#define ELFDATA2LSB 1       /* 2's complement, little endian */
#define PT_LOAD     1       /* Loadable program segment */
#define EM_X86_64   62      /* AMD x86-64 architecture */
#define EM_AARCH64  183     /* ARM aarch64 architecture */

typedef struct
{
  unsigned char e_ident[16];/* Magic number and other info */
  uint16_t    e_type;         /* Object file type */
  uint16_t    e_machine;      /* Architecture */
  uint32_t    e_version;      /* Object file version */
  uint64_t    e_entry;        /* Entry point virtual address */
  uint64_t    e_phoff;        /* Program header table file offset */
  uint64_t    e_shoff;        /* Section header table file offset */
  uint32_t    e_flags;        /* Processor-specific flags */
  uint16_t    e_ehsize;       /* ELF header size in bytes */
  uint16_t    e_phentsize;    /* Program header table entry size */
  uint16_t    e_phnum;        /* Program header table entry count */
  uint16_t    e_shentsize;    /* Section header table entry size */
  uint16_t    e_shnum;        /* Section header table entry count */
  uint16_t    e_shstrndx;     /* Section header string table index */
} Elf64_Ehdr;

typedef struct
{
  uint32_t    p_type;         /* Segment type */
  uint32_t    p_flags;        /* Segment flags */
  uint64_t    p_offset;       /* Segment file offset */
  uint64_t    p_vaddr;        /* Segment virtual address */
  uint64_t    p_paddr;        /* Segment physical address */
  uint64_t    p_filesz;       /* Segment size in file */
  uint64_t    p_memsz;        /* Segment size in memory */
  uint64_t    p_align;        /* Segment alignment */
} Elf64_Phdr;

typedef struct
{
  uint32_t    sh_name;        /* Section name (string tbl index) */
  uint32_t    sh_type;        /* Section type */
  uint64_t    sh_flags;       /* Section flags */
  uint64_t    sh_addr;        /* Section virtual addr at execution */
  uint64_t    sh_offset;      /* Section file offset */
  uint64_t    sh_size;        /* Section size in bytes */
  uint32_t    sh_link;        /* Link to another section */
  uint32_t    sh_info;        /* Additional section information */
  uint64_t    sh_addralign;   /* Section alignment */
  uint64_t    sh_entsize;     /* Entry size if section holds table */
} Elf64_Shdr;

typedef struct
{
  uint32_t    st_name;        /* Symbol name (string tbl index) */
  uint8_t     st_info;        /* Symbol type and binding */
  uint8_t     st_other;       /* Symbol visibility */
  uint16_t    st_shndx;       /* Section index */
  uint64_t    st_value;       /* Symbol value */
  uint64_t    st_size;        /* Symbol size */
} Elf64_Sym;

/*** PE32+ defines and structs ***/
#define MZ_MAGIC                    0x5a4d      /* "MZ" */
#define PE_MAGIC                    0x00004550  /* "PE\0\0" */
#define IMAGE_FILE_MACHINE_AMD64    0x8664      /* AMD x86_64 architecture */
#define IMAGE_FILE_MACHINE_ARM64    0xaa64      /* ARM aarch64 architecture */
#define PE_OPT_MAGIC_PE32PLUS       0x020b      /* PE32+ format */
typedef struct
{
  uint16_t magic;         /* MZ magic */
  uint16_t reserved[29];  /* reserved */
  uint32_t peaddr;        /* address of pe header */
} mz_hdr;

typedef struct {
  uint32_t magic;         /* PE magic */
  uint16_t machine;       /* machine type */
  uint16_t sections;      /* number of sections */
  uint32_t timestamp;     /* time_t */
  uint32_t sym_table;     /* symbol table offset */
  uint32_t numsym;        /* number of symbols */
  uint16_t opt_hdr_size;  /* size of optional header */
  uint16_t flags;         /* flags */
  uint16_t file_type;     /* file type, PE32PLUS magic */
  uint8_t  ld_major;      /* linker major version */
  uint8_t  ld_minor;      /* linker minor version */
  uint32_t text_size;     /* size of text section(s) */
  uint32_t data_size;     /* size of data section(s) */
  uint32_t bss_size;      /* size of bss section(s) */
  int32_t entry_point;    /* file offset of entry point */
  int32_t code_base;      /* relative code addr in ram */
} pe_hdr;

typedef struct {
  uint32_t iszero;        /* if this is not zero, then iszero+nameoffs gives UTF-8 string */
  uint32_t nameoffs;
  int32_t value;          /* value of the symbol */
  uint16_t section;       /* section it belongs to */
  uint16_t type;          /* symbol type */
  uint8_t storclass;      /* storage class */
  uint8_t auxsyms;        /* number of pe_sym records following */
} pe_sym;

/**
 * Check if kernel is conforming with BOOTBOOT
 */
void checkkernel(char *fn)
{
    unsigned char *data = readfileall(fn);
    Elf64_Ehdr *ehdr=(Elf64_Ehdr *)(data);
    Elf64_Phdr *phdr;
    Elf64_Shdr *shdr, *strt, *sym_sh = NULL, *str_sh = NULL;
    Elf64_Sym *sym = NULL, *s;
    pe_hdr *pehdr;
    pe_sym *ps;
    uint32_t i, n = 0, bss = 0, strsz = 0, syment = 0, ma, fa;
    uint64_t core_ptr = 0, core_size = 0, core_addr = 0, entrypoint = 0, mm_addr = 0, fb_addr = 0, bb_addr = 0, env_addr = 0;
    char *strtable, *name;
    if(!data) {
        fprintf(stderr,"mkimg: unable to read %s\n",fn);
        exit(1);
    }
    pehdr=(pe_hdr*)(data + ((mz_hdr*)(data))->peaddr);
    printf("File format: ");
    if((!memcmp(ehdr->e_ident,ELFMAG,SELFMAG)||!memcmp(ehdr->e_ident,"OS/Z",4)) &&
        ehdr->e_ident[EI_CLASS]==ELFCLASS64 && ehdr->e_ident[EI_DATA]==ELFDATA2LSB) {
        printf("ELF64\nArchitecture: %s\n", ehdr->e_machine==EM_AARCH64 ? "AArch64" : (ehdr->e_machine==EM_X86_64 ?
            "x86_64" : "invalid"));
        if(ehdr->e_machine != EM_AARCH64 && ehdr->e_machine != EM_X86_64) return;
        if(ehdr->e_machine == EM_AARCH64) { ma = 2*1024*1024-1; fa = 4095; } else { ma = 4095; fa = 2*1024*1024-1; }
        phdr=(Elf64_Phdr *)((uint8_t *)ehdr+ehdr->e_phoff);
        for(i=0;i<ehdr->e_phnum;i++){
            if(phdr->p_type==PT_LOAD) {
                n++;
                core_ptr = phdr->p_offset;
                core_size = phdr->p_filesz + (ehdr->e_type==3?0x4000:0);
                bss = phdr->p_memsz - core_size;
                core_addr = phdr->p_vaddr;
                entrypoint = ehdr->e_entry;
                break;
            }
            phdr=(Elf64_Phdr *)((uint8_t *)phdr+ehdr->e_phentsize);
        }
        printf("Load segment: %08lx size %ldK offs %lx ", core_addr, (core_size + bss + 1024)/1024, core_ptr);
        if(n != 1) { printf("more than one load segment\n"); return; }
        if((core_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(core_addr & 4095) { printf("not page aligned\n"); return; }
        if(core_size + bss > 16 * 1024 * 1024) { printf("bigger than 16M\n"); return; }
        printf("OK\nEntry point:  %08lx ", entrypoint);
        if(entrypoint < core_addr || entrypoint > core_addr+core_size) { printf("not in text segment\n"); return; }
        printf("OK\n");
        if(ehdr->e_shoff > 0) {
            shdr = (Elf64_Shdr *)((uint8_t *)ehdr + ehdr->e_shoff);
            strt = (Elf64_Shdr *)((uint8_t *)shdr+(uint64_t)ehdr->e_shstrndx*(uint64_t)ehdr->e_shentsize);
            strtable = (char *)ehdr + strt->sh_offset;
            for(i = 0; i < ehdr->e_shnum; i++){
                /* checking shdr->sh_type is not enough, there can be multiple SHT_STRTAB records... */
                if(!memcmp(strtable + shdr->sh_name, ".symtab", 8)) sym_sh = shdr;
                if(!memcmp(strtable + shdr->sh_name, ".strtab", 8)) str_sh = shdr;
                shdr = (Elf64_Shdr *)((uint8_t *)shdr + ehdr->e_shentsize);
            }
            if(str_sh && sym_sh) {
                strtable = (char *)ehdr + str_sh->sh_offset; strsz = str_sh->sh_size;
                sym = (Elf64_Sym *)((uint8_t*)ehdr + sym_sh->sh_offset); syment = sym_sh->sh_entsize;
                if(str_sh->sh_offset && strsz > 0 && sym_sh->sh_offset && syment > 0)
                    for(s = sym, i = 0; i<(strtable-(char*)sym)/syment && s->st_name < strsz; i++, s++) {
                        if(!memcmp(strtable + s->st_name, "bootboot", 9)) bb_addr = s->st_value;
                        if(!memcmp(strtable + s->st_name, "environment", 12)) env_addr = s->st_value;
                        if(!memcmp(strtable + s->st_name, "mmio", 4)) mm_addr = s->st_value;
                        if(!memcmp(strtable + s->st_name, "fb", 3)) fb_addr = s->st_value;
                    }
            } else printf("No symbols found\n");
        } else printf("No section table found\n");
    } else
    if(((mz_hdr*)(data))->magic==MZ_MAGIC && ((mz_hdr*)(data))->peaddr<65536 && pehdr->magic == PE_MAGIC &&
        pehdr->file_type == PE_OPT_MAGIC_PE32PLUS) {
        printf("PE32+\nArchitecture: %s\n", pehdr->machine == IMAGE_FILE_MACHINE_ARM64 ? "AArch64" : (
            pehdr->machine == IMAGE_FILE_MACHINE_AMD64 ? "x86_64" : "invalid"));
        if(pehdr->machine != IMAGE_FILE_MACHINE_ARM64 && pehdr->machine != IMAGE_FILE_MACHINE_AMD64) return;
        if(pehdr->machine == IMAGE_FILE_MACHINE_ARM64) { ma = 2*1024*1024-1; fa = 4095; } else { ma = 4095; fa = 2*1024*1024-1; }
        core_size = (pehdr->entry_point-pehdr->code_base) + pehdr->text_size + pehdr->data_size;
        bss = pehdr->bss_size;
        core_addr = (int64_t)pehdr->code_base;
        entrypoint = (int64_t)pehdr->entry_point;
        printf("Load segment: %08lx size %ldK offs %lx ", core_addr, (core_size + bss + 1024)/1024, core_ptr);
        if((core_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(core_addr & 4095) { printf("not page aligned\n"); return; }
        if(core_size + bss > 16 * 1024 * 1024) { printf("bigger than 16M\n"); return; }
        printf("OK\nEntry point:  %08lx ", entrypoint);
        if(entrypoint < core_addr || entrypoint > core_addr+pehdr->text_size) { printf("not in text segment\n"); return; }
        printf("OK\n");
        if(pehdr->sym_table > 0 && pehdr->numsym > 0) {
            strtable = (char *)pehdr + pehdr->sym_table + pehdr->numsym * 18 + 4;
            for(i = 0; i < pehdr->numsym; i++) {
                ps = (pe_sym*)((uint8_t *)pehdr + pehdr->sym_table + i * 18);
                name = !ps->iszero ? (char*)&ps->iszero : strtable + ps->nameoffs;
                if(!memcmp(name, "bootboot", 9)) bb_addr = (int64_t)ps->value;
                if(!memcmp(name, "environment", 12)) env_addr = (int64_t)ps->value;
                if(!memcmp(name, "mmio", 4)) mm_addr = (int64_t)ps->value;
                if(!memcmp(name, "fb", 3)) fb_addr = (int64_t)ps->value;
                i += ps->auxsyms;
            }
        } else printf("No symbols found\n");
    } else {
        printf("invalid\n");
        return;
    }
    if(!mm_addr && !fb_addr && !bb_addr && !env_addr) {
        printf("\nComplies with BOOTBOOT Protocol Level 1, must use valid static addresses\n");
        return;
    }
    if(mm_addr) {
        printf("mmio:         %08lx ", mm_addr);
        if((mm_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(mm_addr & ma) { printf("not properly aligned\n"); return; }
        printf("OK\n");
    }
    if(fb_addr) {
        printf("fb:           %08lx ", fb_addr);
        if((fb_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(fb_addr & fa) { printf("not properly aligned\n"); return; }
        printf("OK\n");
    }
    if(bb_addr) {
        printf("bootboot:     %08lx ", bb_addr);
        if((bb_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(bb_addr & 4095) { printf("not page aligned\n"); return; }
        printf("OK\n");
    }
    if(env_addr) {
        printf("environment:  %08lx ", env_addr);
        if((env_addr >> 30) != 0x3FFFFFFFF) { printf("not in the higher half top -1G\n"); return; }
        if(env_addr & 4095) { printf("not page aligned\n"); return; }
        printf("OK\n");
    }
    printf("\nComplies with BOOTBOOT Protocol Level %s2, valid dynamic addresses\n",
        (!mm_addr || mm_addr == 0xfffffffff8000000) && (!fb_addr || fb_addr == 0xfffffffffc000000) &&
        (!bb_addr || bb_addr == 0xffffffffffe00000) && (!env_addr || env_addr == 0xffffffffffe01000) &&
        core_addr == 0xffffffffffe02000 && core_size + bss < 2*1024*1024 - 256*1024 - 2*4096 ? "1 and " : "");
}

/**
 * Main entry point
 */
int main(int argc, char **argv)
{
    if(argc < 2 || argv[1]==NULL || !strcmp(argv[1],"help") || (strcmp(argv[1], "rom") && strcmp(argv[1], "check") && argc < 4) ||
        (!strcmp(argv[1], "check") && argc < 3)) {
        printf( "BOOTBOOT mkimg utility - bztsrc@gitlab\n\nUsage:\n"
                "  ./mkimg disk <disk image size in megabytes> <disk image name>\n"
                "  ./mkimg <fat16|fat32> <boot partition size in megabytes> <directory>\n"
                "  ./mkimg rom\n"
                "  ./mkimg check <kernel>\n\n"
                "Creates a hybrid disk / cdrom image from bootpart.bin, or initrd.rom from initrd.bin.\n"
                "It can also create bootpart.bin from the contents of a directory in a portable way.\n"
                "With check you can validate an ELF or PE executable for being BOOTBOOT compatible.\n");
        exit(0);
    }
    t = time(NULL);
    ts = gmtime(&t);

    if(!strcmp(argv[1], "check"))
        checkkernel(argv[2]);
    else
    if(!strcmp(argv[1], "rom"))
        initrdrom();
    else
    if(!memcmp(argv[1], "fat", 3))
        createfat(atoi(argv[1]+3), atoi(argv[2])*1024*1024, argv[3]);
    else
        createdisk(atoi(argv[2])*1024*1024, argv[3]);
    return 0;
}
