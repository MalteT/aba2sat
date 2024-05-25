#!/usr/bin/env python3

import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns

def read_and_visualize(csv_file):
    # Read the CSV file
    df = pd.read_csv(csv_file)

    # Display the first few rows of the dataframe
    print(df.head())

    # Identify all the properties (assuming they are all columns except for some timings)
    properties = [col for col in df.columns if col != 'speedup' and col != 'time' and col != 'stddev']

    # Pairplot to see general pairwise relationships, may help to understand the overall relationship between properties and runtime
    sns.pairplot(df)
    plt.suptitle('Pairplot of Properties and Runtime', y=1.02)
    plt.show()

    # Create scatter plots for each property against runtime
    for prop in properties:
        plt.figure(figsize=(10, 6))
        sns.scatterplot(x=df[prop], y=df['speedup'])
        plt.title(f'Impact of {prop} on Speedup')
        plt.xlabel(prop)
        plt.ylabel('Speedup')

    # Create box plots for categorical properties if any (e.g., difficulty level or type) against runtime
    for prop in properties:
        if df[prop].dtype == 'object':
            plt.figure(figsize=(10, 6))
            sns.boxplot(x=df[prop], y=df['speedup'])
            plt.title(f'Impact of {prop} on Speedup')
            plt.xlabel(prop)
            plt.ylabel('Speedup')

    plt.show()

# Example usage
csv_file = 'all.csv'  # Replace with your actual CSV file path
read_and_visualize(csv_file)
