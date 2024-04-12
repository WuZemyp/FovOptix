use alvr_common::{SlidingWindowAverage, HEAD_ID, LEFT_HAND_ID, RIGHT_HAND_ID};
use alvr_events::{EventType, GraphStatistics, Statistics};
use alvr_packets::ClientStatistics;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant, self}, f64::{NAN, INFINITY}, 
};

use std::error::Error;
use std::fs::OpenOptions;
use std::io::prelude::*;
use csv::Writer;
use chrono::{Utc, TimeZone, Local, format::{strftime, StrftimeItems}};

use crate::{BITRATE_MANAGER,INTER_ARRIVAL_MANGER,TRENDLINE_MANGER,AIMD_MANGER,RateControlInput_MANGER,BitrateEstimator_MANGER,webrtc_gcc_apply::BandwidthUsage,
webrtc_gcc_apply::RateControlState};
const FULL_REPORT_INTERVAL: Duration = Duration::from_millis(500);

pub struct HistoryFrame {
    target_timestamp: Duration,
    tracking_received: Instant,
    frame_present: Instant,
    frame_composed: Instant,
    frame_encoded: Instant,
    total_pipeline_latency: Duration,
    frame_send_timestamp:i64,
    total_packets_belong:i64,
    total_size_for_this_frame:usize,
    reported:bool,//wz repeat
    last_repeat_game_latency:Duration,//wz repeat
    save_flag:bool,
}

impl Default for HistoryFrame {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            target_timestamp: Duration::ZERO,
            tracking_received: now,
            frame_present: now,
            frame_composed: now,
            frame_encoded: now,
            total_pipeline_latency: Duration::ZERO,
            frame_send_timestamp:Utc::now().timestamp_micros(),
            total_packets_belong:0,
            total_size_for_this_frame:0,
            reported:false,//wz repeat
            last_repeat_game_latency:Duration::ZERO,//wz repeat
            save_flag:false,
        }
    }
}


fn write_latency_to_csv(filename: &str, latency_values: [String; 48]) -> Result<(), Box<dyn Error>> {

    let mut file = OpenOptions::new().write(true).append(true).open(filename)?;
    let mut writer = Writer::from_writer(file);

    // Write the latency strings in the next row
    writer.write_record(&[
        &latency_values[0],
        &latency_values[1],
        &latency_values[2],
        &latency_values[3],
        &latency_values[4],
        &latency_values[5],
        &latency_values[6],
        &latency_values[7],
        &latency_values[8],
        &latency_values[9],

        &latency_values[10],
        &latency_values[11],
        &latency_values[12],
        &latency_values[13],
        &latency_values[14],
        &latency_values[15],
        &latency_values[16],
        &latency_values[17],
        &latency_values[18],
        &latency_values[19],
        &latency_values[20],
        &latency_values[21],
        &latency_values[22],
        &latency_values[23],
        &latency_values[24],
        &latency_values[25],
        &latency_values[26],
        &latency_values[27],
        &latency_values[28],
        &latency_values[29],
        &latency_values[30],
        &latency_values[31],
        &latency_values[32],
        &latency_values[33],
        &latency_values[34],
        &latency_values[35],
        &latency_values[36],
        &latency_values[37],
        &latency_values[38],
        &latency_values[39],
        &latency_values[40],
        &latency_values[41],
        &latency_values[42],
        &latency_values[43],
        &latency_values[44],
        &latency_values[45],
        &latency_values[46],
        &latency_values[47],






    ])?;

    Ok(())
}

pub struct StatisticsManager {
    history_buffer: VecDeque<HistoryFrame>,
    max_history_size: usize,
    last_full_report_instant: Instant,
    last_frame_present_instant: Instant,
    last_frame_present_interval: Duration,
    video_packets_total: usize,
    video_packets_partial_sum: usize,
    video_bytes_total: usize,
    video_bytes_partial_sum: usize,
    packets_lost_total: usize,
    packets_lost_partial_sum: usize,
    battery_gauges: HashMap<u64, f32>,
    steamvr_pipeline_latency: Duration,
    total_pipeline_latency_average: SlidingWindowAverage<Duration>,
    last_vsync_time: Instant,
    frame_interval: Duration,

    packet_loss_partial_sum: i32,
    video_bytes_sended_for_webrtc:usize,
    time_instant_for_webrtc:Instant,
    prev_target_bitrate_inrtc:f64,
    last_frame_send_timestamp:i64,
    last_frame_arrival_timestamp:i64,
    pub data_hist:VecDeque<dataPoint>,
    pub smooth_delta:f64,
    pub accumulate_delta:f64,
    pub first_data_send_timestamp:i64,
    pub gradient:f64,
    pub state:u64,
    

}
impl StatisticsManager {
    // history size used to calculate average total pipeline latency
    pub fn new(
        max_history_size: usize,
        nominal_server_frame_interval: Duration,
        steamvr_pipeline_frames: f32,
    ) -> Self {
        Self {
            history_buffer: VecDeque::new(),
            max_history_size,
            last_full_report_instant: Instant::now(),
            last_frame_present_instant: Instant::now(),
            last_frame_present_interval: Duration::ZERO,
            video_packets_total: 0,
            video_packets_partial_sum: 0,
            video_bytes_total: 0,
            video_bytes_partial_sum: 0,
            packets_lost_total: 0,
            packets_lost_partial_sum: 0,
            battery_gauges: HashMap::new(),
            steamvr_pipeline_latency: Duration::from_secs_f32(
                steamvr_pipeline_frames * nominal_server_frame_interval.as_secs_f32(),
            ),
            total_pipeline_latency_average: SlidingWindowAverage::new(
                Duration::ZERO,
                max_history_size,
            ),
            last_vsync_time: Instant::now(),
            frame_interval: nominal_server_frame_interval,

            packet_loss_partial_sum:0,
            video_bytes_sended_for_webrtc:0,
            time_instant_for_webrtc:Instant::now(),
            prev_target_bitrate_inrtc:AIMD_MANGER.lock().current_bitrate_,
            last_frame_send_timestamp:0,
            last_frame_arrival_timestamp:0,
            data_hist:VecDeque::new(),
            smooth_delta:0.0,
            accumulate_delta:0.0,
            first_data_send_timestamp:-1,
            gradient:0.0,
            state:1,

        }
    }

    pub fn report_tracking_received(&mut self, target_timestamp: Duration) {
        if !self
            .history_buffer
            .iter()
            .any(|frame| frame.target_timestamp == target_timestamp)
        {
            self.history_buffer.push_front(HistoryFrame {
                target_timestamp,
                tracking_received: Instant::now(),
                ..Default::default()
            });
        }

        if self.history_buffer.len() > self.max_history_size {
            self.history_buffer.pop_back();
        }
    }

    pub fn report_frame_present(&mut self, target_timestamp: Duration, offset: Duration) {
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            let now = Instant::now() - offset;

            self.last_frame_present_interval =
                now.saturating_duration_since(self.last_frame_present_instant);
            self.last_frame_present_instant = now;

            frame.frame_present = now;
        }
    }

    pub fn report_frame_composed(&mut self, target_timestamp: Duration, offset: Duration) {
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            frame.frame_composed = Instant::now() - offset;
        }
    }

    pub fn report_frame_encoded(&mut self, target_timestamp: Duration) {
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            frame.frame_encoded = Instant::now();
        }
    }
    pub fn report_save(&mut self, target_timestamp: Duration) {
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            frame.save_flag=true;
        }
    }
    pub fn report_video_packet(&mut self, bytes_count: usize,target_timestamp: Duration) {
        self.video_packets_total += 1;
        self.video_packets_partial_sum += 1;
        self.video_bytes_total += bytes_count;
        self.video_bytes_partial_sum += bytes_count;
        self.video_bytes_sended_for_webrtc+=bytes_count;
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            frame.total_packets_belong+=1;
            frame.total_size_for_this_frame=bytes_count;//同一个frame一直在重发，这里没有把所有重发的size算到这个frame上，只算最后的。
        }
    }
    pub fn report_send_timestamp(&mut self,target_timestamp: Duration)
    {
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == target_timestamp)
        {
            frame.frame_send_timestamp = Utc::now().timestamp_micros();
        }

    }

    pub fn report_packet_loss(&mut self) {
        self.packets_lost_total += 1;
        self.packets_lost_partial_sum += 1;
    }

    pub fn report_battery(&mut self, device_id: u64, gauge_value: f32) {
        *self.battery_gauges.entry(device_id).or_default() = gauge_value;
    }

    // Called every frame. Some statistics are reported once every frame
    // Returns network latency
    pub fn report_statistics(&mut self, client_stats: ClientStatistics,current_bitrate:u64,bandwidth:u64) -> Duration {
        //在这里接收clientstatistics中如果有plr则output到csvfile中
        if let Some(frame) = self
            .history_buffer
            .iter_mut()
            .find(|frame| frame.target_timestamp == client_stats.target_timestamp)
        {
            
            
            if self.first_data_send_timestamp==-1{
                self.first_data_send_timestamp=frame.frame_send_timestamp;
            }
            let mut send_delta_ms=0.0;
            let mut recv_delta_ms=0.0;
            if self.last_frame_send_timestamp!=0{
                send_delta_ms=(frame.frame_send_timestamp-self.last_frame_send_timestamp) as f64 *0.001;
                recv_delta_ms=(client_stats.frame_arrival_timestamp-self.last_frame_arrival_timestamp) as f64 *0.001;
            } 
            self.last_frame_send_timestamp=frame.frame_send_timestamp;
            self.last_frame_arrival_timestamp=client_stats.frame_arrival_timestamp;
            
            let mut timestamp_delta=send_delta_ms.to_string();//i64::default();
            let mut arrival_time_delta_ms=recv_delta_ms.to_string();//i64::default();
            let mut packet_size_delta=i64::default();
            let mut network_estimate= "".to_string();
            let mut threshold_c= "".to_string();
            let mut m_trend= "".to_string();
            let mut target_bitrate_bps_inrtc= "".to_string();
            let mut current_state="".to_string();
            let mut next_state="".to_string();
            let mut difference=0.0;
            let delta_flag=true;//INTER_ARRIVAL_MANGER.lock().ComputeDeltas((frame.frame_send_timestamp as f64 *0.001) as i64, (client_stats.frame_arrival_timestamp as f64 *0.001)as i64, (Utc::now().timestamp_micros() as f64 *0.001) as i64, frame.total_size_for_this_frame, &mut timestamp_delta, &mut arrival_time_delta_ms, &mut packet_size_delta);
            let mut b_s=f32::default();
            let mut bitrate_estimate_rtc=f64::default();
            if delta_flag{
                //TRENDLINE_MANGER.lock().UpdateTrendline(arrival_time_delta_ms as f64, timestamp_delta as f64, (frame.frame_send_timestamp as f64 *0.001) as i64, (client_stats.frame_arrival_timestamp as f64 *0.001)as i64, frame.total_size_for_this_frame as i64);
                TRENDLINE_MANGER.lock().UpdateTrendline(recv_delta_ms, send_delta_ms, (frame.frame_send_timestamp as f64 *0.001) as i64, (client_stats.frame_arrival_timestamp as f64 *0.001)as i64, frame.total_size_for_this_frame as i64);
                
                threshold_c=TRENDLINE_MANGER.lock().current_threshold_for_testing.to_string();
                m_trend=TRENDLINE_MANGER.lock().current_trend_for_testing.to_string();
                if TRENDLINE_MANGER.lock().hypothesis_==BandwidthUsage::kBwNormal{
                    network_estimate="normal".to_string();
                    RateControlInput_MANGER.lock().bw_state=BandwidthUsage::kBwNormal;
                }else if TRENDLINE_MANGER.lock().hypothesis_==BandwidthUsage::kBwOverusing{
                    network_estimate="overuse".to_string();
                    RateControlInput_MANGER.lock().bw_state=BandwidthUsage::kBwOverusing;
                }else{
                    network_estimate="underuse".to_string();
                    RateControlInput_MANGER.lock().bw_state=BandwidthUsage::kBwUnderusing;
                }
                
                if AIMD_MANGER.lock().rate_control_state_==RateControlState::kRcHold{
                    current_state="hold".to_string();
                    
                }else if AIMD_MANGER.lock().rate_control_state_==RateControlState::kRcDecrease{
                    current_state="decrease".to_string();
                }else{
                    current_state="increase".to_string();
                }
                
                BitrateEstimator_MANGER.lock().Update((client_stats.frame_arrival_timestamp as f64*0.001) as i64,frame.total_size_for_this_frame,false);
                if BitrateEstimator_MANGER.lock().bitrate().is_some(){
                    bitrate_estimate_rtc=BitrateEstimator_MANGER.lock().bitrate().unwrap();
                    RateControlInput_MANGER.lock().estimated_throughput=Some(bitrate_estimate_rtc);
                 }
                 //else{
                //     bitrate_estimate_rtc=self.video_bytes_sended_for_webrtc as f64/Instant::now().saturating_duration_since(self.time_instant_for_webrtc).as_secs_f64()*8./1e6 *1024.0*1024.0;
                // }
                //b_s=self.video_bytes_sended_for_webrtc as f32/Instant::now().saturating_duration_since(self.time_instant_for_webrtc).as_secs_f32()*8./1e6 *1024.0*1024.0;
                self.video_bytes_sended_for_webrtc=0;
                self.time_instant_for_webrtc=Instant::now();
                // if b_s>1.0 && b_s<(200*1024*1024) as f32{
                //     RateControlInput_MANGER.lock().estimated_throughput=Some(b_s as f64);
                // }
                
                //看ratecontrolinput中estimate bandwidth值是不是0，现在不加不减，或者是multiple赠的时候出问题了
                target_bitrate_bps_inrtc=(AIMD_MANGER.lock().Update(&RateControlInput_MANGER.lock(), (Utc::now().timestamp_micros() as f64 *0.001) as i64)/1024.0/1024.0).to_string();
                difference=AIMD_MANGER.lock().current_bitrate_-self.prev_target_bitrate_inrtc;
                let prev_target_bitrate=self.prev_target_bitrate_inrtc;
                self.prev_target_bitrate_inrtc=AIMD_MANGER.lock().current_bitrate_;

                let mut delta=AIMD_MANGER.lock().current_bitrate_/72./8.+prev_target_bitrate/72./8.-2.*(frame.total_size_for_this_frame as f64);
                let normalize=delta/(AIMD_MANGER.lock().current_bitrate_/72./8.);
                AIMD_MANGER.lock().normalize_delta=normalize;
                self.accumulate_delta+=delta;
                self.smooth_delta=0.9*self.smooth_delta+0.1*self.accumulate_delta;
                self.data_hist.push_back(dataPoint::new((frame.frame_send_timestamp-self.first_data_send_timestamp) as f64, self.smooth_delta));
                if self.data_hist.len()>20{
                    self.data_hist.pop_front();
                }
                
                if self.data_hist.len()==20{
                    self.gradient=FitLine_Get_Gradient(&self.data_hist).unwrap_or(self.gradient);
                }

                if AIMD_MANGER.lock().rate_control_state_==RateControlState::kRcHold{
                    next_state="hold".to_string();
                    
                }else if AIMD_MANGER.lock().rate_control_state_==RateControlState::kRcDecrease{
                    next_state="decrease".to_string();
                }else{
                    next_state="increase".to_string();
                }
            }
            
            frame.total_pipeline_latency = client_stats.total_pipeline_latency;
            //self.packet_loss_partial_sum+=client_stats.plr;
            let mut game_time_latency = frame
                .frame_present
                .saturating_duration_since(frame.tracking_received);//wz repeat

            let server_compositor_latency = frame
                .frame_composed
                .saturating_duration_since(frame.frame_present);

            let encoder_latency = frame
                .frame_encoded
                .saturating_duration_since(frame.frame_composed);

            // The network latency cannot be estiamed directly. It is what's left of the total
            // latency after subtracting all other latency intervals. In particular it contains the
            // transport latency of the tracking packet and the interval between the first video
            // packet is sent and the last video packet is received for a specific frame.
            // For safety, use saturating_sub to avoid a crash if for some reason the network
            // latency is miscalculated as negative.
            let network_latency = frame.total_pipeline_latency.saturating_sub(
                game_time_latency
                    + server_compositor_latency
                    + encoder_latency
                    + client_stats.video_decode
                    + client_stats.video_decoder_queue
                    + client_stats.rendering
                    + client_stats.vsync_queue,
            );

            let client_fps = 1.0
                / client_stats
                    .frame_interval
                    .max(Duration::from_millis(1))
                    .as_secs_f32();
            let server_fps = 1.0
                / self
                    .last_frame_present_interval
                    .max(Duration::from_millis(1))
                    .as_secs_f32();
            
            let mut plr="".to_string();
            let mut bitrate_statistics="".to_string();//bitrate bps
            let mut total_packets_send="".to_string();//total packets
            let mut average_packet_size="".to_string();//average packet size
            if self.last_full_report_instant + FULL_REPORT_INTERVAL < Instant::now() {
                self.last_full_report_instant += FULL_REPORT_INTERVAL;

                let interval_secs = FULL_REPORT_INTERVAL.as_secs_f32();
                

                alvr_events::send_event(EventType::Statistics(Statistics {
                    video_packets_total: self.video_packets_total,
                    video_packets_per_sec: (self.video_packets_partial_sum as f32 / interval_secs)
                        as _,
                    video_mbytes_total: (self.video_bytes_total as f32 / 1e6) as usize,
                    video_mbits_per_sec: self.video_bytes_partial_sum as f32 / interval_secs * 8.
                        / 1e6,
                    total_latency_ms: client_stats.total_pipeline_latency.as_secs_f32() * 1000.,
                    network_latency_ms: network_latency.as_secs_f32() * 1000.,
                    encode_latency_ms: encoder_latency.as_secs_f32() * 1000.,
                    decode_latency_ms: client_stats.video_decode.as_secs_f32() * 1000.,
                    packets_lost_total: self.packets_lost_total,
                    packets_lost_per_sec: (self.packets_lost_partial_sum as f32 / interval_secs)
                        as _,
                    client_fps: client_fps as _,
                    server_fps: server_fps as _,
                    battery_hmd: (self
                        .battery_gauges
                        .get(&HEAD_ID)
                        .cloned()
                        .unwrap_or_default()
                        * 100.) as _,
                    battery_left: (self
                        .battery_gauges
                        .get(&LEFT_HAND_ID)
                        .cloned()
                        .unwrap_or_default()
                        * 100.) as _,
                    battery_right: (self
                        .battery_gauges
                        .get(&RIGHT_HAND_ID)
                        .cloned()
                        .unwrap_or_default()
                        * 100.) as _,
                }));
                bitrate_statistics=(self.video_bytes_partial_sum as f32 / FULL_REPORT_INTERVAL.as_secs_f32() * 8./ 1e6).to_string();//bitrate mbps
                //let b_s=(self.video_bytes_partial_sum as f32 / FULL_REPORT_INTERVAL.as_secs_f32() * 8./ 1e6);
                
                
                total_packets_send=self.video_packets_partial_sum.to_string();
                if self.video_packets_partial_sum!=0
                {
                    average_packet_size=(self.video_bytes_partial_sum/self.video_packets_partial_sum).to_string();
                    //plr=(self.packet_loss_partial_sum as f64/self.video_packets_partial_sum as f64).to_string();
                }
                else {
                    average_packet_size="".to_string();
                    //plr="no data".to_string();
                }

                self.video_packets_partial_sum = 0;
                self.video_bytes_partial_sum = 0;
                self.packets_lost_partial_sum = 0;
                self.packet_loss_partial_sum=0;
            }
            //wz repeat
            
            if frame.reported{
                game_time_latency=game_time_latency.saturating_sub(frame.last_repeat_game_latency);
                
                frame.total_pipeline_latency=frame.total_pipeline_latency.saturating_sub(frame.last_repeat_game_latency);
                //frame.total_pipeline_latency-=frame.last_repeat_game_latency;
            }
            frame.reported=true;
            frame.last_repeat_game_latency+=game_time_latency;
            //wz repeat

            // todo: use target timestamp in nanoseconds. the dashboard needs to use the first
            // timestamp as the graph time origin.

            //let mut file=OpenOptions::new().append(true).write(true).open(r"C:\Users\13513\Desktop\extract_data1.txt").unwrap();

            let mut interval_trackingReceived_framePresentInVirtualDevice=(game_time_latency.as_secs_f32()*1000.).to_string();//game latency
            let mut interval_framePresentInVirtualDevice_frameComposited=(server_compositor_latency.as_secs_f32()*1000.).to_string();//composite latency
            let mut interval_frameComposited_VideoEncoded=(encoder_latency.as_secs_f32() * 1000.).to_string();//encode latency
            let mut interval_VideoReceivedByClient_VideoDecoded=(client_stats.video_decode.as_secs_f32() * 1000.).to_string();//decode latency
            let mut interval_network=((network_latency.as_secs_f32()*1000.).to_string());//network latency(interval_trackingsend_trackingreceived+interval_encodedVideoSend_encodedVideoReceived)
            let mut interval_total_pipeline=(frame.total_pipeline_latency.as_secs_f32() * 1000.).to_string();//total pipeline latency wz repeat
            let mut target_bitrate=(current_bitrate/1024/1024).to_string();//target bitrate
            let mut shard_loss_rate=client_stats.shard_loss_rate.to_string();
            if client_stats.flag_plr{
                plr=(client_stats.plr*100.0).to_string();//pakcet loss rate record every second
            }
            //let mut bdw=bandwidth.to_string();
            //let mut plr=(client_stats.plr*100.0).to_string()+"%"+"\t";//pakcet loss rate record every second

            //let mut bdw=(current_bitrate as f64)/(1.0-client_stats.plr)/((network_latency.as_secs_f32()*1000.)as f64);//bitrate/(1-plr)/network latency
            let mut frame_send_timestamp=frame.frame_send_timestamp.to_string();
            let mut frame_arrive_timestamp=client_stats.frame_arrival_timestamp.to_string();
            let mut total_size_for_this_frame=frame.total_size_for_this_frame.to_string();
            let mut total_packets_for_this_frame=frame.total_packets_belong.to_string();
            let mut timestamp_delta_string="".to_string();
            let mut arrival_time_delta_ms_string="".to_string();
            let mut packet_size_delta_string="".to_string();
            let mut bitrate_pass_to_webrtc="".to_string();
            if delta_flag{
                timestamp_delta_string=timestamp_delta.to_string();
                arrival_time_delta_ms_string=arrival_time_delta_ms.to_string();
                packet_size_delta_string=packet_size_delta.to_string();
                bitrate_pass_to_webrtc=b_s.to_string();
                
            }
            let mut last_decrease=AIMD_MANGER.lock().last_decrease_.unwrap().to_string();
            let mut link_upper=AIMD_MANGER.lock().link_capacity_.UpperBound().to_string();
            let mut link_lower=AIMD_MANGER.lock().link_capacity_.LowerBound().to_string();
            let mut link_has_estimate=AIMD_MANGER.lock().link_capacity_.has_estimate().to_string();
            let mut link_estimate="".to_string();
            if AIMD_MANGER.lock().link_capacity_.has_estimate(){
                link_estimate=AIMD_MANGER.lock().link_capacity_.estimate().to_string();
            }
            let mut link_devia=AIMD_MANGER.lock().link_capacity_.deviation_kbps_.to_string();
            let mut test_thr=AIMD_MANGER.lock().esitmate_thr_testing.to_string();
            let mut input_thr="".to_string();
            if RateControlInput_MANGER.lock().estimated_throughput.is_some(){
                input_thr=RateControlInput_MANGER.lock().estimated_throughput.unwrap().to_string();
            }
            let mut delta=AIMD_MANGER.lock().current_bitrate_/72./8.-frame.total_size_for_this_frame as f64;
            let mut gradient_c=self.gradient;
            if gradient_c>0.&& delta>0.{
                self.state=2;
            }else if gradient_c<0.&&delta<0.{
                self.state=0;
            }else{
                self.state=1;
            }
            if (frame.total_pipeline_latency.as_secs_f32() * 1000.>100.)&&!frame.save_flag{//wz repeat
                self.state=0;
            }
            AIMD_MANGER.lock().flag_for_qp=self.state;
            
            let experiment_target_timestamp=Local::now().format("%Y%m%d_%H%M%S").to_string();
            let latency_strings=[interval_trackingReceived_framePresentInVirtualDevice,interval_framePresentInVirtualDevice_frameComposited,interval_frameComposited_VideoEncoded,interval_VideoReceivedByClient_VideoDecoded,interval_network,(client_stats.video_decoder_queue.as_secs_f32()*1000.).to_string(),(client_stats.rendering.as_secs_f32()*1000.).to_string(),(client_stats.vsync_queue.as_secs_f32()*1000.).to_string(),interval_total_pipeline,target_bitrate,plr,bitrate_statistics,total_packets_send,average_packet_size,shard_loss_rate,frame.total_size_for_this_frame.to_string(),frame_send_timestamp,frame_arrive_timestamp,total_size_for_this_frame,total_packets_for_this_frame,timestamp_delta_string,arrival_time_delta_ms_string,packet_size_delta_string,network_estimate,threshold_c,m_trend,target_bitrate_bps_inrtc,current_state,next_state,bitrate_pass_to_webrtc,difference.to_string(),last_decrease,link_lower,link_upper,link_has_estimate,link_estimate,link_devia,test_thr,input_thr,bitrate_estimate_rtc.to_string(),gradient_c.to_string(),delta.to_string(),self.state.to_string(),frame.target_timestamp.as_nanos().to_string(),client_stats.flag_debug.to_string(),client_stats.is_idr.to_string(),AIMD_MANGER.lock().normalize_delta.to_string(),experiment_target_timestamp];
            write_latency_to_csv("C:\\AT\\QP_manager\\build\\alvr_streamer_windows\\statistics.csv", latency_strings);

            alvr_events::send_event(EventType::GraphStatistics(GraphStatistics {
                total_pipeline_latency_s: frame.total_pipeline_latency.as_secs_f32(),//wz repeat
                game_time_s: game_time_latency.as_secs_f32(),
                server_compositor_s: server_compositor_latency.as_secs_f32(),
                encoder_s: encoder_latency.as_secs_f32(),
                network_s: network_latency.as_secs_f32(),
                decoder_s: client_stats.video_decode.as_secs_f32(),
                decoder_queue_s: client_stats.video_decoder_queue.as_secs_f32(),
                client_compositor_s: client_stats.rendering.as_secs_f32(),
                vsync_queue_s: client_stats.vsync_queue.as_secs_f32(),
                client_fps,
                server_fps,
            }));

            network_latency
        } else {
            Duration::ZERO
        }
    }

    pub fn tracker_pose_time_offset(&self) -> Duration {
        // This is the opposite of the client's StatisticsManager::tracker_prediction_offset().
        self.steamvr_pipeline_latency
            .saturating_sub(self.total_pipeline_latency_average.get_average())
    }

    // NB: this call is non-blocking, waiting should be done externally
    pub fn duration_until_next_vsync(&mut self) -> Duration {
        let now = Instant::now();

        // update the last vsync if it's too old
        while self.last_vsync_time + self.frame_interval < now {
            self.last_vsync_time += self.frame_interval;
        }

        (self.last_vsync_time + self.frame_interval).saturating_duration_since(now)
    }


    

    
}
pub struct dataPoint{
    pub send_time_ms:f64,
    pub delta_value:f64,
}
impl dataPoint{
    fn new(send_time_ms:f64,delta_value:f64)->Self{
        Self { 
            send_time_ms,
            delta_value,
        }
    }
}

pub fn FitLine_Get_Gradient(packets:& VecDeque<dataPoint>)->Option<f64> {
    if packets.len()>2 {
              // Compute the "center of mass".
          let mut sum_x = 0.0;
          let mut sum_y = 0.0;
          for packet in packets {
              sum_x += packet.send_time_ms;
              sum_y += packet.delta_value;
          }
          let mut x_avg = sum_x / packets.len() as f64;
          let mut y_avg = sum_y / packets.len() as f64;
          // Compute the slope k = \sum (x_i-x_avg)(y_i-y_avg) / \sum (x_i-x_avg)^2
          let mut numerator = 0.0;
          let mut denominator = 0.0;
          for packet in packets {
              let mut  x = packet.send_time_ms;
              let mut y = packet.delta_value;
              numerator += (x - x_avg) * (y - y_avg);
              denominator += (x - x_avg) * (x - x_avg);
          }
          if denominator == 0.0{
              return Option::None;
          }
              
          return Some(numerator / denominator);
    }else{
      return Option::None;
    }
  }


