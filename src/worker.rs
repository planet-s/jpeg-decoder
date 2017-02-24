use decoder::MAX_COMPONENTS;
use idct::dequantize_and_idct_block;
use parser::Component;
use std::mem;
use std::sync::Arc;

pub struct RowData {
    pub index: usize,
    pub component: Component,
    pub quantization_table: Arc<[u16; 64]>,
}

pub struct Worker {
    offsets: Box<[usize]>,
    results: Vec<Vec<u8>>,
    components: Vec<Option<Component>>,
    quantization_tables: Vec<Option<Arc<[u16; 64]>>>
}

impl Worker {
    pub fn new() -> Self {
        Worker {
            offsets: vec![0; MAX_COMPONENTS].into_boxed_slice(),
            results: vec![Vec::new(); MAX_COMPONENTS],
            components: vec![None; MAX_COMPONENTS],
            quantization_tables: vec![None; MAX_COMPONENTS],
        }
    }

    pub fn start(&mut self, data: RowData) {
        let offsets = &mut self.offsets;
        let results = &mut self.results;
        let components = &mut self.components;
        let quantization_tables = &mut self.quantization_tables;

        assert!(results[data.index].is_empty());

        offsets[data.index] = 0;
        results[data.index].resize(data.component.block_size.width as usize * data.component.block_size.height as usize * 64, 0u8);
        components[data.index] = Some(data.component);
        quantization_tables[data.index] = Some(data.quantization_table);
    }

    pub fn append_row(&mut self, index: usize, data: Vec<i16>) {
        // Convert coefficients from a MCU row to samples.

        let offsets = &mut self.offsets;
        let results = &mut self.results;
        let components = &mut self.components;
        let quantization_tables = &mut self.quantization_tables;

        let component = components[index].as_ref().unwrap();
        let quantization_table = quantization_tables[index].as_ref().unwrap();
        let block_count = component.block_size.width as usize * component.vertical_sampling_factor as usize;
        let line_stride = component.block_size.width as usize * 8;

        assert_eq!(data.len(), block_count * 64);

        for i in 0 .. block_count {
            let x = (i % component.block_size.width as usize) * 8;
            let y = (i / component.block_size.width as usize) * 8;
            dequantize_and_idct_block(&data[i * 64 .. (i + 1) * 64],
                                      quantization_table,
                                      line_stride,
                                      &mut results[index][offsets[index] + y * line_stride + x ..]);
        }

        offsets[index] += data.len();
    }

    pub fn get_result(&mut self, index: usize) -> Vec<u8> {
        mem::replace(&mut self.results[index], Vec::new())
    }
}
