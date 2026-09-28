namespace Braillify.Tests.NetFramework;

using System;
using Microsoft.VisualStudio.TestTools.UnitTesting;

[TestClass]
public sealed class BraillifyTests
{
    [TestMethod]
    public void EncodeToUnicode_SimpleKorean_ReturnsExpectedBraille()
    {
        // Arrange
        const string input = "안녕하세요";
        const string expected = "⠣⠒⠉⠻⠚⠠⠝⠬";

        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.AreEqual(expected, result);
    }

    [TestMethod]
    public void Encode_SimpleKorean_ReturnsNonEmptyByteArray()
    {
        // Arrange
        const string input = "안녕";

        // Act
        var result = Braillify.Encode(input);

        // Assert
        Assert.IsTrue(result.Length > 0);
    }

    [TestMethod]
    public void EncodeToBrailleFont_SimpleKorean_ReturnsNonEmptyString()
    {
        // Arrange
        const string input = "테스트";

        // Act
        var result = Braillify.EncodeToBrailleFont(input);

        // Assert
        Assert.IsFalse(string.IsNullOrEmpty(result));
    }

    [TestMethod]
    public void EncodeToUnicode_NullInput_ThrowsArgumentNullException()
    {
        // Act & Assert
        Assert.ThrowsExactly<ArgumentNullException>(() => Braillify.EncodeToUnicode(null!));
    }

    [TestMethod]
    [DataRow("상상이상의", "⠇⠶⠇⠶⠕⠇⠶⠺")]
    [DataRow("1,000", "⠼⠁⠂⠚⠚⠚")]
    [DataRow("ATM", "⠠⠠⠁⠞⠍")]
    public void EncodeToUnicode_VariousInputs_ReturnsExpectedResults(string input, string expected)
    {
        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.AreEqual(expected, result);
    }

    [TestMethod]
    public void EncodeToUnicode_EmptyString_ReturnsEmptyString()
    {
        // Arrange
        const string input = "";

        // Act
        var result = Braillify.EncodeToUnicode(input);

        // Assert
        Assert.AreEqual(string.Empty, result);
    }

    [TestMethod]
    public void EncodeInContext_Science_ReadsTheUnitFormula()
    {
        Assert.AreEqual("⠴⠏⠠⠕⠠⠓", Braillify.EncodeToUnicode("pOH", "science"));
        Assert.AreEqual("⠴⠏⠠⠠⠕⠓⠲", Braillify.EncodeToUnicode("pOH", "korean"));
        Assert.AreEqual("⠴⠏⠠⠕⠠⠓", Braillify.EncodeToBrailleFont("pOH", "science"));
        Assert.AreEqual(6, Braillify.Encode("pOH", "science").Length);
    }

    [TestMethod]
    public void EncodeInContext_UnknownContext_ThrowsBraillifyException()
    {
        Assert.ThrowsExactly<BraillifyException>(() => Braillify.EncodeToUnicode("pOH", "chemistry"));
    }

    [TestMethod]
    public void EncodeInContext_NullContext_ThrowsArgumentNullException()
    {
        Assert.ThrowsExactly<ArgumentNullException>(() => Braillify.EncodeToUnicode("pOH", null!));
    }

    [TestMethod]
    public void Encode_EmptyString_ReturnsEmptyArray()
    {
        // Arrange
        const string input = "";

        // Act
        var result = Braillify.Encode(input);

        // Assert
        Assert.AreEqual(0, result.Length);
    }
}
