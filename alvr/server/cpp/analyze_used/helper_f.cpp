#include "helper_f.h"

int frame_count = 0;
int save_frame_feq = 1000;

void add_frame_count(){
    frame_count++;
}

int get_frame_count(){
    return frame_count;
}

int get_save_frame_feq(){
    return save_frame_feq;
}

std::string filename_s = "C:\\AT\\QP_manager\\build\\alvr_streamer_windows\\";

std::string get_path_head(){
    return filename_s;
}

void SaveTextureAsBytes(ID3D11DeviceContext* context, ID3D11Texture2D* texture, std::string name)
{
	ID3D11Device* device;
	texture->GetDevice(&device);
    // Get texture description
    D3D11_TEXTURE2D_DESC desc;
    texture->GetDesc(&desc);

    // Create staging texture
    D3D11_TEXTURE2D_DESC stagingDesc = desc;
    stagingDesc.Usage = D3D11_USAGE_STAGING;
    stagingDesc.BindFlags = 0;
    stagingDesc.CPUAccessFlags = D3D11_CPU_ACCESS_READ;
    ID3D11Texture2D* stagingTexture;
    device->CreateTexture2D(&stagingDesc, nullptr, &stagingTexture);

    // Copy texture to staging texture
    context->CopyResource(stagingTexture, texture);

    // Map staging texture to CPU memory
    D3D11_MAPPED_SUBRESOURCE mappedResource;
    context->Map(stagingTexture, 0, D3D11_MAP_READ, 0, &mappedResource);

    // Write texture to byte file
	const char* filename = (filename_s+name).c_str();
    std::ofstream file(filename, std::ios::out | std::ios::binary);
    file.write((char*)mappedResource.pData, mappedResource.DepthPitch);

    // Unmap staging texture
    context->Unmap(stagingTexture, 0);

    // Release resources
    stagingTexture->Release();
}