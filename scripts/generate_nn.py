#!/usr/bin/env python3

import torch
import torch.nn as nn
import torch.optim as optim
import psutil
from torch.utils.data import DataLoader
from torchvision import datasets, transforms


# Define the neural network architecture
class NeuralNetwork(nn.Module):
    def __init__(self):
        super(NeuralNetwork, self).__init__()
        self.fc1 = nn.Linear(28*28, 16)  # Input layer
        self.fc2 = nn.Linear(16, 16)     # Hidden layer 1
        self.fc3 = nn.Linear(16, 16)     # Hidden layer 2
        self.fc4 = nn.Linear(16, 16)     # Hidden layer 3
        self.fc5 = nn.Linear(16, 10)     # Output layer

    def forward(self, x):
        x = x.view(-1, 28*28)  # Flatten the input images
        x = torch.relu(self.fc1(x))
        x = torch.relu(self.fc2(x))
        x = torch.relu(self.fc3(x))
        x = torch.relu(self.fc4(x))
        x = self.fc5(x)
        return x

def run_training():
    # Get the number of physical cores
    num_physical_cores = psutil.cpu_count(logical=False)

    torch.set_num_threads(num_physical_cores)

    # Load MNIST dataset
    transform = transforms.Compose([transforms.ToTensor(), transforms.Normalize((0.5,), (0.5,))])
    train_dataset = datasets.MNIST(root='./data', train=True, download=True, transform=transform)
    train_loader = DataLoader(train_dataset, batch_size=2000, shuffle=True)

    # Initialize the neural network
    model = NeuralNetwork()

    # Define loss function and optimizer
    criterion = nn.CrossEntropyLoss()
    optimizer = optim.SGD(model.parameters(), lr=0.01)

    # Training loop
    epochs = 10
    for epoch in range(epochs):
        running_loss = 0.0
        for i, data in enumerate(train_loader, 0):
            inputs, labels = data
            optimizer.zero_grad()
            outputs = model(inputs)
            loss = criterion(outputs, labels)
            loss.backward()
            optimizer.step()

            running_loss += loss.item()
            if i % 10 == 9:    # Print every 10 mini-batches
                print('[%3d, %5d] loss: %.3f' % (epoch + 1, i + 1, running_loss / 10))
                running_loss = 0.0

    torch.save(model.state_dict(), 'data/model.pth')
    print('Finished Training')

if __name__ == '__main__':
    run_training()
