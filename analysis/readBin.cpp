#include <stdio.h>
#include <stdlib.h>
#include <string.h>

extern "C"{
#include <libavcodec/avcodec.h>
#include <libavutil/imgutils.h>
#include <libavutil/opt.h>
}

int main(int argc, char *argv[])
{
    const AVCodec *codec;
    AVCodecContext *codec_ctx;
    AVFrame *frame;
    AVPacket packet;
    int ret, got_frame = 0;
    FILE *infile, *outfile;
    uint8_t *buf;
    size_t buf_size;

    // Open the input file containing the encoded data
    infile = fopen("enc_2000.h264", "rb");
    if (!infile) {
        fprintf(stderr, "Failed to open input file\n");
        return 1;
    }

    // Read the encoded data into a buffer
    fseek(infile, 0, SEEK_END);
    buf_size = ftell(infile);
    fseek(infile, 0, SEEK_SET);
    buf = (uint8_t *)malloc(buf_size);
    if (!buf) {
        fprintf(stderr, "Failed to allocate memory\n");
        return 1;
    }
    fread(buf, 1, buf_size, infile);
    fclose(infile);

    // Initialize the ffmpeg library
    //av_register_all();

    // Initialize the ffmpeg decoder
    codec = avcodec_find_decoder(AV_CODEC_ID_H264);
    codec_ctx = avcodec_alloc_context3(codec);
    if (!codec_ctx) {
        fprintf(stderr, "Failed to allocate codec context\n");
        return 1;
    }
    ret = avcodec_open2(codec_ctx, codec, NULL);
    if (ret < 0) {
        fprintf(stderr, "Failed to open codec\n");
        return 1;
    }

    // Allocate memory for the decoded frames
    frame = av_frame_alloc();
    if (!frame) {
        fprintf(stderr, "Failed to allocate frame\n");
        return 1;
    }

    // Decode the encoded data
    av_init_packet(&packet);
    packet.data = buf;
    packet.size = buf_size;
    while (packet.size > 0) {
        ret = avcodec_send_packet(codec_ctx, &packet);
        if (ret < 0) {
            fprintf(stderr, "Error sending packet to decoder\n");
            return 1;
        }
        while (ret >= 0) {
            ret = avcodec_receive_frame(codec_ctx, frame);
            if (ret == AVERROR(EAGAIN) || ret == AVERROR_EOF) {
                break;
            }
            if (ret < 0) {
                fprintf(stderr, "Error receiving frame from decoder\n");
                return 1;
            }
            // Process or display the decoded frame
        }
        packet.data += ret;
        packet.size -= ret;
    }

    // Cleanup
    avcodec_close(codec_ctx);
    avcodec_free_context(&codec_ctx);
    av_frame_free(&frame);
    free(buf);

    return 0;
}