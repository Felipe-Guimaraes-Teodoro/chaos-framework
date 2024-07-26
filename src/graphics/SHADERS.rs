
pub static DEFAULT_VS: &str = r#"
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoord;
layout (location = 3) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;
uniform vec3 color;

out vec4 fColor;
out vec3 Normal;
out vec3 FragPos;
out vec2 TexCoord; // Pass texture coordinates to the fragment shader

void main() {
    gl_Position = proj * view * model * vec4(aPos, 1.0);
    fColor = vec4(color, 1.0);
    TexCoord = aTexCoord; // Pass texture coordinates
    FragPos = vec3(model * vec4(aPos, 1.0));
    Normal = mat3(transpose(inverse(model))) * aNormal;  
}
"#;

pub static DEFAULT_FS: &str = r#"
#version 330 core
out vec4 FragColor;

in vec4 fColor;
in vec2 TexCoord;
in vec3 Normal;
in vec3 FragPos;  

uniform vec3 lightColor[256];
uniform vec3 lightPos[256];
uniform vec3 viewPos;

uniform int has_texture;
uniform int num_lights;

uniform sampler2D texture1;

void main()
{
    vec4 texColor = fColor;

    if (has_texture == 1) {
       texColor = texture(texture1, TexCoord) * fColor;
    }

    vec3 ambientStrength = vec3(0.1); 
    vec3 specularStrength = vec3(0.5);

    vec3 result = vec3(0.0);

    for (int i = 0; i < num_lights; ++i) {
        vec3 lightDir = normalize(lightPos[i] - FragPos); 
        float distance = length(lightPos[i] - FragPos);

        float constant_attenuation = 1.0;
        float linear_attenuation = 0.045;
        float quadratic_attenuation = 0.016;

        float attenuation = 1.0 / (constant_attenuation + linear_attenuation * distance + quadratic_attenuation * distance * distance);

        vec3 norm = normalize(Normal);
        float diff = max(dot(norm, lightDir), 0.0);
        vec3 diffuse = diff * lightColor[i] * attenuation;

        vec3 viewDir = normalize(viewPos - FragPos);
        vec3 reflectDir = reflect(-lightDir, norm); 

        float spec = pow(max(dot(viewDir, reflectDir), 0.0), 32);
        vec3 specular = specularStrength * spec * lightColor[i] * attenuation;

        result += ((ambientStrength / num_lights) + diffuse + specular) * texColor.rgb;
    }

    FragColor = vec4(result, texColor.a);
}
"#;


pub static RUSSIMP_VS: &str = r#"
#version 430 core

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec2 tex;
layout(location = 3) in ivec4 boneIds; 
layout(location = 4) in vec4 weights;

uniform mat4 proj;
uniform mat4 view;
uniform mat4 model;
uniform vec3 color;

const int MAX_BONES = 100;
uniform mat4 finalBonesMatrices[MAX_BONES];

out vec2 TexCoord;
out vec3 Normal;
out vec3 FragPos;
out vec4 fColor;

void main()
{
    mat4 boneTransform = mat4(0.0);
    for (int i = 0; i < 4; i++) {
        if (boneIds[i] != -1) {
            boneTransform += finalBonesMatrices[boneIds[i]] * weights[i];
        }
    }
    
    vec4 f_pos = boneTransform * vec4(pos, 1.0);
    gl_Position = proj * view * model * f_pos;

    TexCoord = tex;
    fColor = vec4(color, 1.0);
    FragPos = vec3(model * f_pos);

    // Correcting the normal transformation
    Normal = mat3(transpose(inverse(model))) * norm;  
}

"#;

use crate::Shader;

use std::sync::LazyLock;

pub static DEFAULT_SHADER: LazyLock<Shader> = LazyLock::new(|| {
    Shader::new_pipeline(DEFAULT_VS, DEFAULT_FS)
});


pub static RUSSIMP_SHADER: LazyLock<Shader> = LazyLock::new(|| {
    Shader::new_pipeline(RUSSIMP_VS, DEFAULT_FS)
});

