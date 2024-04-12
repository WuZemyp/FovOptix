#include <fstream>
#include <iostream>
#include <vector>


bool checkStartCodeAndNALUnitTypes(const std::string& filename) {
    std::ifstream infile(filename, std::ios::binary);

    if (!infile) {
        std::cerr << "Error: Unable to open file " << filename << std::endl;
        return false;
    }

    // Read the entire file into a buffer
    std::vector<uint8_t> buffer(std::istreambuf_iterator<char>(infile), {});

    size_t pos = 0;
    bool foundSPS = false;
    bool foundPPS = false;
    bool foundIDR = false;

    // Search for NAL units with the expected types
    while (pos < buffer.size()) {
        // Find the start code (00 00 01) or (00 00 00 01)
        if (buffer[pos] == 0x00 && buffer[pos + 1] == 0x00 && (buffer[pos + 2] == 0x01 || (buffer[pos + 2] == 0x00 && buffer[pos + 3] == 0x01))) {
            uint8_t nal_unit_type = buffer[pos + (buffer[pos + 2] == 0x01 ? 3 : 4)] & 0x1F;

            switch (nal_unit_type) {
                case 7: // SPS
                    foundSPS = true;
                    break;
                case 8: // PPS
                    foundPPS = true;
                    break;
                case 5: // IDR
                    foundIDR = true;
                    break;
                default:
                    break;
            }
        }
        
        std::cout << foundSPS << foundPPS << foundIDR << std::endl;
        if (foundSPS && foundPPS && foundIDR) {
            return true;
        }

        pos++;
    }

    return false;
}

int main(){
    checkStartCodeAndNALUnitTypes("enc_2000.h264");
}