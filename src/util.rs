use std::{
    fmt::{Debug, Display},
    ops::{Index, IndexMut},
};

#[derive(Clone)]
pub struct Array2D<T> {
    pub data: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T> Array2D<T>
where
    T: Clone + Default + Copy,
{
    pub fn new(width: usize, height: usize) -> Self {
        Array2D {
            data: vec![Default::default(); width * height],
            width,
            height,
        }
    }

    pub fn fill(data: T, width: usize, height: usize) -> Self {
        Array2D {
            data: vec![data; width * height],
            width,
            height,
        }
    }

    pub fn zero(&mut self) {
        self.data
            .copy_from_slice(&[Default::default()].repeat(self.width * self.height));
    }

    pub fn reset(&mut self, value: T) {
        self.data
            .copy_from_slice(&[value].repeat(self.width * self.height));
    }
}

impl<T> Array2D<T> {
    pub fn from_vec(value: Vec<T>, width: usize, height: usize) -> Self {
        Array2D {
            data: value,
            width,
            height,
        }
    }
}

impl<T> Index<(usize, usize)> for Array2D<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        if index.0 >= self.width || index.1 >= self.height {
            panic!(
                "Index out of bounds: ({},{}) not in [0,{})x[0,{}))",
                index.0, index.1, self.width, self.height
            );
        }
        &self.data[index.1 * self.width + index.0]
    }
}

impl<T> IndexMut<(usize, usize)> for Array2D<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        if index.0 >= self.width || index.1 >= self.height {
            panic!(
                "Index out of bounds: ({},{}) not in [0,{})x[0,{}))",
                index.0, index.1, self.width, self.height
            );
        }
        &mut self.data[index.1 * self.width + index.0]
    }
}

impl<T> Display for Array2D<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            writeln!(f)?;
            for x in 0..self.width {
                write!(f, "{:^9}", format!("{:+.4} ", self[(x, y)]))?;
            }
        }
        Ok(())
    }
}

impl<T> Debug for Array2D<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)?;
        Ok(())
    }
}
