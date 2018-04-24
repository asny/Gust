use attribute;
use std::string::String;
use glm;

#[derive(Debug)]
pub enum Error {
    FailedToFindCustomAttribute {message: String},
    Attribute(attribute::Error)
}

impl From<attribute::Error> for Error {
    fn from(other: attribute::Error) -> Self {
        Error::Attribute(other)
    }
}

pub struct Mesh {
    no_vertices: usize,
    indices: Vec<u32>,
    positions: attribute::Attribute,
    custom_attributes: Vec<attribute::Attribute>
}


impl Mesh
{
    pub fn create(indices: Vec<u32>, positions: Vec<f32>) -> Result<Mesh, Error>
    {
        let no_vertices = positions.len()/3;
        let mut p = Vec::with_capacity(no_vertices);
        for vid in 0..no_vertices {
            p.push(glm::vec3(positions[vid * 3], positions[vid * 3 + 1], positions[vid * 3 + 2]));
        }

        let position_attribute = attribute::Attribute::create_vec3_attribute("Position", p)?;
        let mesh = Mesh { no_vertices, indices, positions: position_attribute, custom_attributes: Vec::new() };
        Ok(mesh)
    }

    pub fn positions(&self) -> &attribute::Attribute
    {
        &self.positions
    }

    pub fn indices(&self) -> &Vec<u32>
    {
        &self.indices
    }

    pub fn no_vertices(&self) -> usize
    {
        self.no_vertices
    }

    pub fn get(&self, name: &str) -> Result<&attribute::Attribute, Error>
    {
        for attribute in self.custom_attributes.iter() {
            if attribute.name() == name
            {
                return Ok(attribute)
            }
        }
        Err(Error::FailedToFindCustomAttribute{message: format!("Failed to find {} attribute", name)})
    }

    pub fn add_custom_attribute(&mut self, name: &str, data: Vec<glm::Vec3>) -> Result<(), Error>
    {
        let custom_attribute = attribute::Attribute::create_vec3_attribute(name, data)?;
        self.custom_attributes.push(custom_attribute);
        Ok(())
    }
}
