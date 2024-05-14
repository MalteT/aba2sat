#!/usr/bin/env python3

import torch
from PIL import Image
import torchvision.transforms as transforms
from generate_nn import NeuralNetwork

model = NeuralNetwork()
model.load_state_dict(torch.load('data/model.pth'))
model.eval()

# Define a transform to convert the image to tensor and normalize it
# MNIST images are usually transformed to tensors and normalized with a mean of 0.1307 and std of 0.3081
transform = transforms.Compose([
    transforms.Grayscale(num_output_channels=1), # Convert to grayscale if the image is in color
    transforms.Resize((28, 28)),  # Resize image to 28x28 pixels
    transforms.ToTensor(),  # Convert to PyTorch Tensor
    transforms.Normalize((0.1307,), (0.3081,)),  # Normalize pixel values
])

# Load the image
image = Image.open('test.png')

# Apply the transform to the image
image = transform(image)

# Add an extra batch dimension since PyTorch treats all inputs as batches
image = torch.unsqueeze(image, 0)

with torch.no_grad():
    output = model(image)
    predicted_class = output.argmax(dim = 1).item()
    print('Predicted class:', predicted_class)
