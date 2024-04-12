#include <iostream>
#include <fstream> 
#include "../platform/win32/NvEncoderD3D11.h"

#ifndef HELPERS_H
#define HELPERS_H

extern int frame_count;
extern int save_frame_feq;
extern std::string filename_s;

void add_frame_count();
int get_frame_count();
int get_save_frame_feq();
std::string get_path_head();
void SaveTextureAsBytes(ID3D11DeviceContext* context, ID3D11Texture2D* texture, std::string name);

#endif